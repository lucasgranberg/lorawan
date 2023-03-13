pub mod radio;
pub mod timer;
use radio::PhyRxTx;
use rand_core::RngCore;

use crate::DR;

use self::timer::Timer;

pub trait Device {
    type Timer: Timer;
    type PhyRxTx: PhyRxTx;
    type Rng: RngCore;
    fn timer(&mut self) -> &mut Self::Timer;
    fn radio(&mut self) -> &mut Self::PhyRxTx;
    fn rng(&mut self) -> &mut Self::Rng;
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
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    Radio,
}
