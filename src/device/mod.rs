//! Wrapper for all necessary functionality implemented by calling code.

pub mod non_volatile_store;
pub mod radio;
pub mod radio_buffer;
pub mod rng;
pub mod timer;

use radio::Radio;
use rng::Rng;
use timer::Timer;

use crate::mac::types::{Configuration, Credentials, Storable, DR};

use self::non_volatile_store::NonVolatileStore;

#[cfg(not(feature = "defmt"))]
macro_rules! trace {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            let _ = ($( & $x ),*);
        }
    };
}
#[cfg(feature = "defmt")]
use defmt::trace;

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
    /// Persist information required to maintain communication with a network server through end device power cycles.
    fn persist_to_non_volatile(
        &mut self,
        configuration: &Configuration,
        credentials: &Credentials,
    ) -> Result<(), <Self::NonVolatileStore as NonVolatileStore>::Error> {
        let storable = Storable {
            rx1_data_rate_offset: configuration.rx1_data_rate_offset,
            rx_delay: configuration.rx_delay,
            rx2_data_rate: configuration.rx2_data_rate,
            rx2_frequency: configuration.rx2_frequency,
            dev_nonce: credentials.dev_nonce,
        };
        if let Ok(old_storable) = self.non_volatile_store().load() {
            if storable != old_storable {
                trace!("overwrite {} {}", old_storable, storable);
                self.non_volatile_store().save(storable)?;
            } else {
                trace!("nothing changed {}", storable);
            }
        } else {
            trace!("Save fresh {}", storable);
            self.non_volatile_store().save(storable)?;
        }
        Ok(())
    }

    /// Restore information required to maintain end device communication with a network server.
    fn hydrate_from_non_volatile(
        &mut self,
        app_eui: [u8; 8],
        dev_eui: [u8; 8],
        app_key: [u8; 16],
    ) -> Result<(Configuration, Credentials), <Self::NonVolatileStore as NonVolatileStore>::Error>
    {
        let storable: Storable = self.non_volatile_store().load()?;
        let configuration = Configuration {
            rx1_data_rate_offset: storable.rx1_data_rate_offset,
            rx_delay: storable.rx_delay,
            rx2_data_rate: storable.rx2_data_rate,
            rx2_frequency: storable.rx2_frequency,
            ..Default::default()
        };
        let mut credentials = Credentials::new(app_eui, dev_eui, app_key);
        credentials.dev_nonce = storable.dev_nonce;
        Ok((configuration, credentials))
    }
    /// Create a DevStatusAns response to a network server specifying battery level as directed by the caller.
    fn battery_level(&self) -> Option<f32> {
        None
    }
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
    /// Get the preferred channel block index for join requests as indicated by the caller.
    /// For both dynamic and fixed plans, there are a maximum of 80 channels: 10 channel blocks
    /// of 8 channels each.  Therefore, valid indexes are 0 through 9.
    fn preferred_join_channel_block_index() -> usize {
        0
    }
}
