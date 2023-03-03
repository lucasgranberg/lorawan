use crate::device::radio::types::{Bandwidth, SpreadingFactor};

trait Region: crate::mac::Region {}

pub mod eu868;
// This datarate type is used internally for defining bandwidth/sf per region
#[derive(Debug, Clone)]
pub(crate) struct Datarate {
    bandwidth: Bandwidth,
    spreading_factor: SpreadingFactor,
}
