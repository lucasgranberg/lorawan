//! Join request/accept scheduler to establish a session between the network and end device.

use defmt::trace;
use futures::pin_mut;

use crate::device::radio::Radio;
use crate::device::rng::Rng;
use crate::device::timer::Timer;
use crate::device::Device;
use crate::mac::region::channel_plan::{Channel, ChannelPlan};
use crate::mac::region::Region;
use crate::mac::types::Window;
use crate::mac::Frame;
use crate::mac::Mac;

/// Run the scheduler.
pub async fn run_scheduler<R: Region, C: ChannelPlan<R> + Default, D: Device + defmt::Format>(
    mac: &mut Mac<R, C>,
    device: &mut D,
) -> Result<(), crate::Error<D>> {
    let mut radio_buffer = Default::default();
    while !mac.is_joined() {
        defmt::info!("JOINING");
        mac.prepare_for_join_request(device, &mut radio_buffer)?;
        let channels = mac.get_send_channels(device, Frame::Join)?;
        for chn in channels {
            if let Some(channel) = chn {
                let tx_data_rate = R::override_ul_data_rate_if_necessary(
                    mac.tx_data_rate(),
                    Frame::Join,
                    channel.get_ul_frequency(),
                );
                let tx_config = mac.create_tx_config(Frame::Join, &channel, tx_data_rate)?;
                trace!("tx config {:?}", tx_config);
                let _ms = device
                    .radio()
                    .tx(tx_config, radio_buffer.as_ref())
                    .await
                    .map_err(crate::device::Error::Radio)?;

                let windows = mac.get_rx_windows(Frame::Join);
                device.timer().reset();
                let open_rx1_fut = device
                    .timer()
                    .at(windows.get_open(&Window::_1) as u64)
                    .map_err(crate::device::Error::Timer)?;
                let open_rx2_fut = device
                    .timer()
                    .at(windows.get_open(&Window::_2) as u64)
                    .map_err(crate::device::Error::Timer)?;
                pin_mut!(open_rx2_fut);
                open_rx1_fut.await;

                {
                    radio_buffer.clear();
                    let rf_config =
                        mac.create_rf_config(&Frame::Join, &Window::_1, tx_data_rate, &channel)?;
                    match device.radio().rx(rf_config, Some(1), radio_buffer.as_raw_slice()).await {
                        Ok((num_read, _rx_quality)) => {
                            radio_buffer.inc(num_read);
                            match mac.handle_join_accept(device, &mut radio_buffer) {
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
                            defmt::error!("Rx1 radio error: {:?}", err);
                        }
                    }
                }

                open_rx2_fut.await;

                {
                    radio_buffer.clear();
                    let rf_config =
                        mac.create_rf_config(&Frame::Join, &Window::_2, tx_data_rate, &channel)?;
                    match device.radio().rx(rf_config, Some(1), radio_buffer.as_raw_slice()).await {
                        Ok((num_read, _rx_quality)) => {
                            radio_buffer.inc(num_read);
                            match mac.handle_join_accept(device, &mut radio_buffer) {
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
                            defmt::error!("Rx2 radio error: {:?}", err);
                        }
                    }
                }
            }

            // Delay for a random amount of time between 1 and 2 seconds ???
            let random = device.rng().next_u32().map_err(crate::device::Error::Rng)?;
            let delay_ms = 1000 + (random % 1000);
            device.timer().reset();
            let delay_fut =
                device.timer().at(delay_ms as u64).map_err(crate::device::Error::Timer)?;
            delay_fut.await;
        }

        // Add a delay before working through a random channel from each channel block again, or implement backoff ???
    }
    Ok(())
}
