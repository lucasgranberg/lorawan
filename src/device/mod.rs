pub mod credentials_store;
pub mod radio;
pub mod radio_buffer;
pub mod rng;
pub mod timer;

use radio::Radio;
use rng::Rng;
use timer::Timer;

use crate::DR;

use self::credentials_store::CredentialsStore;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<D>
where
    D: Device,
{
    Timer(<<D as Device>::Timer as Timer>::Error),
    Radio(<<D as Device>::Radio as Radio>::Error),
    Rng(<<D as Device>::Rng as Rng>::Error),
    CredentialsStore(<<D as Device>::CredentialsStore as CredentialsStore>::Error),
    RadioBuffer(radio_buffer::Error),
}
pub trait Device {
    type Timer: Timer;
    type Radio: Radio;
    type Rng: Rng;
    type CredentialsStore: CredentialsStore;
    fn timer(&mut self) -> &mut Self::Timer;
    fn radio(&mut self) -> &mut Self::Radio;
    fn rng(&mut self) -> &mut Self::Rng;
    fn credentials_store(&mut self) -> &mut Self::CredentialsStore;
    fn max_eirp() -> i8;
    fn handle_device_time(&mut self, _seconds: u32, _nano_seconds: u32) {
        // default do nothing
    }
    fn handle_link_check(&mut self, _gateway_count: u8, _margin: u8) {
        // default do nothing
    }

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
    fn adaptive_data_rate_enabled() -> bool;
}
