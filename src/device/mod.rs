//! Wrapper for all necessary functionality implemented by calling code.

pub mod non_volatile_store;
pub mod radio;
pub mod radio_buffer;
pub mod rng;
pub mod timer;

use radio::Radio;
use rng::Rng;
use timer::Timer;

use crate::mac::types::DR;

use self::non_volatile_store::NonVolatileStore;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(missing_docs)]
pub enum Error<D>
where
    D: Device,
{
    Timer(<<D as Device>::Timer as Timer>::Error),
    Radio(<<D as Device>::Radio as Radio>::Error),
    Rng(<<D as Device>::Rng as Rng>::Error),
    NonVolatileStore(<<D as Device>::NonVolatileStore as NonVolatileStore>::Error),
    RadioBuffer(radio_buffer::Error),
}
impl<D> From<Error<D>> for super::Error<D>
where
    D: Device,
{
    fn from(value: Error<D>) -> Self {
        Self::Device(value)
    }
}
pub trait Device {
    type Timer: Timer;
    type Radio: Radio;
    type Rng: Rng;
    type NonVolatileStore: NonVolatileStore;
    fn timer(&mut self) -> &mut Self::Timer;
    fn radio(&mut self) -> &mut Self::Radio;
    fn rng(&mut self) -> &mut Self::Rng;
    fn non_volatile_store(&mut self) -> &mut Self::NonVolatileStore;
    fn max_eirp() -> u8;
    fn handle_device_time(&mut self, _seconds: u32, _nano_seconds: u32) {
        // default do nothing
    }
    fn handle_link_check(&mut self, _gateway_count: u8, _margin: u8) {
        // default do nothing
    }

    fn adaptive_data_rate_enabled(&self) -> bool {
        true
    }

    fn battery_level(&self) -> Option<f32> {
        None
    }
}

pub trait DeviceSpecs {
    fn min_frequency() -> Option<u32> {
        None
    }
    fn max_frequency() -> Option<u32> {
        None
    }
    fn min_data_rate() -> Option<DR> {
        None
    }
    fn max_data_rate() -> Option<DR> {
        None
    }
}
