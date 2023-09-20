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
use crate::mac::types::{RxWindows, Window, DR};
use crate::mac::Frame;
use crate::mac::Mac;

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

        // Temporary stub for RxC values.  ???
        let rxc_channels = mac.get_send_channels(device, Frame::Data)?;
        let rxc_channel = rxc_channels.into_iter().flatten().next().unwrap();
        let rxc_data_rate = R::override_ul_data_rate_if_necessary(
            mac.tx_data_rate(),
            Frame::Data,
            rxc_channel.get_ul_frequency(),
        );
        let class_c_rf_config =
            mac.create_rf_config(&Frame::Data, &Window::_2, rxc_data_rate, &rxc_channel)?;

        let (_timer, radio, uplink_packet_queue) = device.future_generators();

        let mut radio_buffer = Default::default();
        match uplink_vs_rxc(radio, uplink_packet_queue, class_c_rf_config, &mut radio_buffer).await
        {
            (Some(uplink_packet), None) => {
                let _ = class_a_with_rxc(mac, device, uplink_packet, class_c_rf_config).await;
                // handle error ???
            }
            (None, Some(rxc)) => {
                radio_buffer.inc(rxc.0);
                if let Err(err) = mac.handle_downlink(device, &mut radio_buffer, rxc.1).await {
                    defmt::error!("downlink error: {:?}", err);
                }
            }
            _ => {}
        };
    }
}

async fn class_a_with_rxc<R: Region, C: ChannelPlan<R> + Default, D: Device + defmt::Format>(
    mac: &mut Mac<R, C>,
    device: &mut D,
    uplink_packet: PacketBuffer<PACKET_SIZE>,
    class_c_rf_config: RfConfig,
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
    for class_a_channel in channels.into_iter().flatten() {
        let class_a_tx_data_rate = R::override_ul_data_rate_if_necessary(
            mac.tx_data_rate(),
            Frame::Data,
            class_a_channel.get_ul_frequency(),
        );
        let tx_config =
            mac.create_tx_config(Frame::Data, &class_a_channel, class_a_tx_data_rate)?;
        trace!("tx config {:?}", tx_config);
        trans_index += 1;
        let _ms = device
            .radio()
            .tx(tx_config, radio_buffer.as_ref())
            .await
            .map_err(crate::device::Error::Radio)?;

        let windows = mac.get_rx_windows(Frame::Data);
        if !rx1_rx2_vs_rxc(
            mac,
            device,
            class_c_rf_config,
            windows,
            class_a_channel,
            class_a_tx_data_rate,
        )
        .await
            && uplink_packet.confirm_uplink
        {
            defmt::error!("Uplink packet not confirmed");
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

async fn uplink_vs_rxc<L, Q>(
    radio: &mut L,
    uplink_packet_queue: &mut Q,
    class_c_rf_config: RfConfig,
    radio_buffer: &mut RadioBuffer<256>,
) -> (Option<PacketBuffer<PACKET_SIZE>>, Option<(usize, RxQuality)>)
where
    L: Radio,
    Q: PacketQueue,
{
    loop {
        radio_buffer.clear();

        let uplink_packet_queue_fut = uplink_packet_queue.next();
        let rxc_fut = radio.rx(class_c_rf_config, None, radio_buffer.as_raw_slice());

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

async fn rx1_rx2_vs_rxc<R: Region, C: ChannelPlan<R> + Default, D: Device + defmt::Format>(
    mac: &mut Mac<R, C>,
    device: &mut D,
    class_c_rf_config: RfConfig,
    windows: RxWindows,
    class_a_channel: <C as ChannelPlan<R>>::Channel,
    class_a_tx_data_rate: DR,
) -> bool {
    let mut is_rx1_stage = true;
    device.timer().reset();
    let open_rx1_fut = device.timer().at(windows.get_open(&Window::_1) as u64).unwrap(); // error handling ???
    let open_rx2_fut = device.timer().at(windows.get_open(&Window::_2) as u64).unwrap(); // error handling ???
    pin_mut!(open_rx1_fut);
    pin_mut!(open_rx2_fut);
    loop {
        let mut rxc_radio_buffer: RadioBuffer<256> = Default::default();

        let (is_timeout, rxc_option) = {
            let rxc_fut =
                device.radio().rx(class_c_rf_config, None, rxc_radio_buffer.as_raw_slice());
            pin_mut!(rxc_fut);

            if is_rx1_stage {
                match select(open_rx1_fut.as_mut(), rxc_fut).await {
                    Either::Left((_result, _)) => (true, None),
                    Either::Right((result, _)) => match result {
                        Ok(rxc) => (false, Some(rxc)),
                        Err(err) => {
                            defmt::error!("RxC rx error: {:?}", err);
                            (false, None)
                        }
                    },
                }
            } else {
                match select(open_rx2_fut.as_mut(), rxc_fut).await {
                    Either::Left((_result, _)) => (true, None),
                    Either::Right((result, _)) => match result {
                        Ok(rxc) => (false, Some(rxc)),
                        Err(err) => {
                            defmt::error!("RxC rx error: {:?}", err);
                            (false, None)
                        }
                    },
                }
            }
        };

        if !is_timeout && rxc_option.is_none() {
            continue;
        } else if !is_timeout && rxc_option.is_some() {
            let rxc = rxc_option.unwrap();
            rxc_radio_buffer.inc(rxc.0);
            if let Err(err) = mac.handle_downlink(device, &mut rxc_radio_buffer, rxc.1).await {
                defmt::error!("downlink error: {:?}", err);
            }
            continue;
        }

        if is_rx1_stage {
            let rx1_rf_config = match mac.create_rf_config::<D>(
                &Frame::Data,
                &Window::_1,
                class_a_tx_data_rate,
                &class_a_channel,
            ) {
                Ok(rf_config) => rf_config,
                Err(err) => {
                    defmt::error!("Rx1 create_rf_config error: {:?}", err);
                    return false;
                }
            };
            let mut rx1_radio_buffer: RadioBuffer<256> = Default::default();
            match device.radio().rx(rx1_rf_config, Some(1), rx1_radio_buffer.as_raw_slice()).await {
                Ok((num_read, rx_quality)) => {
                    rx1_radio_buffer.inc(num_read);
                    match mac.handle_downlink(device, &mut rx1_radio_buffer, rx_quality).await {
                        Ok(true) => {
                            return true;
                        }
                        Ok(false) => {
                            is_rx1_stage = false;
                        }
                        Err(err) => {
                            defmt::error!("downlink error: {:?}", err);
                            is_rx1_stage = false;
                        }
                    }
                }
                Err(err) => {
                    defmt::error!("Rx1 radio error: {:?}", err);
                    is_rx1_stage = false;
                }
            }
        } else {
            let rx2_rf_config = match mac.create_rf_config::<D>(
                &Frame::Data,
                &Window::_2,
                class_a_tx_data_rate,
                &class_a_channel,
            ) {
                Ok(rf_config) => rf_config,
                Err(err) => {
                    defmt::error!("Rx2 create_rf_config error: {:?}", err);
                    return false;
                }
            };
            let mut rx2_radio_buffer: RadioBuffer<256> = Default::default();
            match device.radio().rx(rx2_rf_config, Some(1), rx2_radio_buffer.as_raw_slice()).await {
                Ok((num_read, rx_quality)) => {
                    rx2_radio_buffer.inc(num_read);
                    match mac.handle_downlink(device, &mut rx2_radio_buffer, rx_quality).await {
                        Ok(true) => {
                            return true;
                        }
                        Ok(false) => {
                            return false;
                        }
                        Err(err) => {
                            defmt::error!("downlink error: {:?}", err);
                            return false;
                        }
                    }
                }
                Err(err) => {
                    defmt::error!("Rx2 radio error: {:?}", err);
                    return false;
                }
            }
        }
    }
}
