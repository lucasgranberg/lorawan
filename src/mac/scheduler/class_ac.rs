//! WIP: Class A/C scheduler for a session between the network and end device.

use defmt::trace;
use futures::future::{select, Either};
use futures::pin_mut;

use crate::device::packet_buffer::PacketBuffer;
use crate::device::packet_queue::{PacketQueue, PACKET_SIZE};
use crate::device::radio::types::{RfConfig, RxQuality};
use crate::device::radio::Radio;
use crate::device::radio_buffer::RadioBuffer;
use crate::device::rng::Rng;
use crate::device::timer::Timer;
use crate::device::Device;
use crate::mac::region::channel_plan::{Channel, ChannelPlan};
use crate::mac::region::Region;
use crate::mac::types::Window;
use crate::mac::Mac;
use crate::mac::{Error, Frame};

/// Run the scheduler for processing associated with the class mode A and C.
pub async fn run_scheduler<R: Region, C: ChannelPlan<R> + Default, D: Device + defmt::Format>(
    mac: &mut Mac<R, C>,
    device: &mut D,
) -> Result<(), crate::Error<D>> {
    loop {
        if !mac.is_joined() {
            match crate::mac::scheduler::join::run_scheduler(mac, device).await {
                Ok(()) => match device.radio().sleep(false).await {
                    Ok(()) => {}
                    Err(e) => defmt::error!("Radio sleep failed with error {:?}", e),
                },
                Err(err) => {
                    return Err(err);
                }
            }
        }

        // Temporary until RxC values are determined ???
        let rxc_channels = mac.get_send_channels(device, Frame::Data)?;
        let rxc_channel = rxc_channels.into_iter().flatten().next().unwrap();
        let rxc_data_rate = R::override_ul_data_rate_if_necessary(
            mac.tx_data_rate(),
            Frame::Data,
            rxc_channel.get_ul_frequency(),
        );
        let rf_config =
            mac.create_rf_config(&Frame::Data, &Window::_2, rxc_data_rate, &rxc_channel)?;

        let (_timer, radio, uplink_packet_queue) = device.future_generators();

        let mut radio_buffer = Default::default();
        match uplink_vs_rxc(radio, uplink_packet_queue, rf_config, &mut radio_buffer).await {
            (Some(uplink_packet), None) => {
                let _ = class_a_with_rxc(mac, device, uplink_packet).await; // handle error ???
            }
            (None, Some(rxc)) => {
                radio_buffer.inc(rxc.0);
                if let Err(err) = mac.handle_downlink(device, &mut radio_buffer, rxc.1).await {
                    defmt::error!("downlink packet queue error: {:?}", err);
                }
            }
            _ => {}
        };
    }
}

async fn uplink_vs_rxc<L, Q>(
    radio: &mut L,
    uplink_packet_queue: &mut Q,
    rf_config: RfConfig,
    radio_buffer: &mut RadioBuffer<256>,
) -> (Option<PacketBuffer<PACKET_SIZE>>, Option<(usize, RxQuality)>)
where
    L: Radio,
    Q: PacketQueue,
{
    loop {
        radio_buffer.clear();

        let uplink_packet_queue_fut = uplink_packet_queue.next();
        let rxc_fut = radio.rx(rf_config, None, radio_buffer.as_raw_slice());

        pin_mut!(uplink_packet_queue_fut);
        pin_mut!(rxc_fut);

        match select(uplink_packet_queue_fut, rxc_fut).await {
            Either::Left((result, _)) => match result {
                Ok(packet) => {
                    return (Some(packet), None);
                }
                Err(err) => {
                    defmt::error!("uplink packet queue error: {:?}", err);
                }
            },
            Either::Right((result, _)) => match result {
                Ok(rx_attr) => {
                    return (None, Some(rx_attr));
                }
                Err(err) => {
                    defmt::error!("RxC rx error: {:?}", err);
                }
            },
        };
    }
}

async fn class_a_with_rxc<R: Region, C: ChannelPlan<R> + Default, D: Device + defmt::Format>(
    mac: &mut Mac<R, C>,
    device: &mut D,
    uplink_packet: PacketBuffer<PACKET_SIZE>,
) -> Result<(), crate::Error<D>> {
    defmt::info!("SENDING");
    let mut trans_index = 0u8;
    let mut radio_buffer = Default::default();
    mac.prepare_for_uplink(
        device,
        &mut radio_buffer,
        uplink_packet.as_ref(),
        1,
        uplink_packet.confirm_uplink,
    )?;
    let channels = mac.get_send_channels(device, Frame::Data)?;
    for channel in channels.into_iter().flatten() {
        let tx_data_rate = R::override_ul_data_rate_if_necessary(
            mac.tx_data_rate(),
            Frame::Data,
            channel.get_ul_frequency(),
        );
        let tx_config = mac.create_tx_config(Frame::Data, &channel, tx_data_rate)?;
        trace!("tx config {:?}", tx_config);
        trans_index += 1;
        let _ms = device
            .radio()
            .tx(tx_config, radio_buffer.as_ref())
            .await
            .map_err(crate::device::Error::Radio)?;

        let windows = mac.get_rx_windows(Frame::Data);
        device.timer().reset();
        let open_rx1_fut = device
            .timer()
            .at(windows.get_open(&Window::_1) as u64)
            .map_err(crate::device::Error::Timer)?;
        let open_rx2_fut = device
            .timer()
            .at(windows.get_open(&Window::_2) as u64)
            .map_err(crate::device::Error::Timer)?;
        let mut skip_rx2 = false;
        pin_mut!(open_rx2_fut);
        open_rx1_fut.await;

        {
            radio_buffer.clear();
            let rf_config =
                mac.create_rf_config(&Frame::Data, &Window::_1, tx_data_rate, &channel)?;
            match device.radio().rx(rf_config, Some(1), radio_buffer.as_raw_slice()).await {
                Ok((num_read, rx_quality)) => {
                    radio_buffer.inc(num_read);
                    match mac.handle_downlink(device, &mut radio_buffer, rx_quality).await {
                        Ok(true) => {
                            skip_rx2 = true;
                        }
                        Ok(false) => {}
                        Err(err) => {
                            return Err(err);
                        }
                    }
                }
                Err(err) => {
                    defmt::trace!("Rx1 radio error: {:?}", err);
                }
            }
        }

        if !skip_rx2 {
            open_rx2_fut.await;

            {
                radio_buffer.clear();
                let rf_config =
                    mac.create_rf_config(&Frame::Data, &Window::_2, tx_data_rate, &channel)?;
                match device.radio().rx(rf_config, Some(1), radio_buffer.as_raw_slice()).await {
                    Ok((num_read, rx_quality)) => {
                        radio_buffer.inc(num_read);
                        match mac.handle_downlink(device, &mut radio_buffer, rx_quality).await {
                            Ok(true) => {
                                break;
                            }
                            Ok(false) => {}
                            Err(err) => {
                                return Err(err);
                            }
                        }
                    }
                    Err(err) => {
                        defmt::trace!("Rx2 radio error: {:?}", err);
                    }
                }
            }
        }
        if uplink_packet.confirm_uplink {
            return Err(crate::Error::Mac(Error::NoResponse));
        }
        if trans_index >= mac.configuration.number_of_transmissions {
            break;
        } else {
            // Delay for a random amount of time between 1 and 2 seconds ???
            let random = device.rng().next_u32().map_err(crate::device::Error::Rng)?;
            let delay_ms = 1000 + (random % 1000);
            device.timer().reset();
            let delay_fut =
                device.timer().at(delay_ms as u64).map_err(crate::device::Error::Timer)?;
            delay_fut.await;
        }
    }

    Ok(())
}
