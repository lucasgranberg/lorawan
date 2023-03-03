pub mod radio;
pub mod timer;
use radio::PhyRxTx;
use rand_core::RngCore;

use self::timer::Timer;

pub trait Device {
    type Timer: Timer;
    type PhyRxTx: PhyRxTx;
    type Rng: RngCore;
    fn timer(&mut self) -> &mut Self::Timer;
    fn radio(&mut self) -> &mut Self::PhyRxTx;
    fn rng(&mut self) -> &mut Self::Rng;
}
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    Radio,
}
