pub mod radio;
pub mod timer;
use radio::PhyRxTx;

use self::timer::Timer;

pub trait Device {
    type Timer: Timer;
    type PhyRxTx: PhyRxTx;
    fn timer(&mut self) -> &mut Self::Timer;
    fn radio(&mut self) -> &mut Self::PhyRxTx;
}
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    Radio,
}
