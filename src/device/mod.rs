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

/// Specification of end device-specific functionality provided by the caller.
pub trait Device {
    /// Timer provided by the calling code.
    type Timer: Timer;
    /// Radio provided by the calling code.
    type Radio: Radio;
    /// Random number generator provided by calling code.
    type Rng: Rng;
    /// Storage capability provided by calling code.
    type NonVolatileStore: NonVolatileStore;

    /// Get the caller-supplied timer implementation.
    fn timer(&mut self) -> &mut Self::Timer;
    /// Get the caller-supplied LoRa radio implementation.
    fn radio(&mut self) -> &mut Self::Radio;
    /// Get the caller-supllied random number generator implementation.
    fn rng(&mut self) -> &mut Self::Rng;
    /// Get the caller-supplied persistence implementation.
    fn non_volatile_store(&mut self) -> &mut Self::NonVolatileStore;
    /// Get the caller-supplied maximum EIRP.
    fn max_eirp() -> u8;
    /// Process the DeviceTimeAns response from a network server as directed by the caller.
    fn handle_device_time(&mut self, _seconds: u32, _nano_seconds: u32) {
        // default do nothing
    }
    /// Process the LinkCheckAns response from a network server as directed by the caller.
    fn handle_link_check(&mut self, _gateway_count: u8, _margin: u8) {
        // default do nothing
    }
    /// Process the LinkADRReq request from a network server as directed by the caller.
    fn adaptive_data_rate_enabled(&self) -> bool {
        true
    }
    /// Create a DevStatusAns response to a network server specifying battery level as directed by the caller.
    fn battery_level(&self) -> Option<f32> {
        None
    }
}

/// Specification of end device-specific limits provided by the caller.
pub trait DeviceSpecs {
    /// Get the minimum frequency supported by a device as indicated by the caller.
    fn min_frequency() -> Option<u32> {
        None
    }
    /// Get the maximum frequency supported by a device as indicated by the caller.
    fn max_frequency() -> Option<u32> {
        None
    }
    /// Get the minimum DR supported by a device as indicated by the caller.
    fn min_data_rate() -> Option<DR> {
        None
    }
    /// Get the maximum DR supported by a device as indicated by the caller.
    fn max_data_rate() -> Option<DR> {
        None
    }
}
