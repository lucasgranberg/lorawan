//! Specification of functionality implemented for each supported LoRaWAN region.

use crate::device::radio::types::{CodingRate, Datarate};
use crate::device::Device;

use self::channel_plan::dynamic::DynamicChannel;

use super::types::{Frame, DR};
pub mod channel_plan;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(missing_docs)]
pub enum Error {
    InvalidTxPower,
    InvalidChannelIndex,
    InvalidChannelMaskCtrl,
    InvalidFrequency,
    DataRateNotSupported(DR),
    UnsupportedRx1DROffset(DR, u8),
    NoValidChannelFound,
    InvalidCfListType,
    CommandNotImplementedForRegion,
    UnsupportedChannelListForRegion,
}
impl<D> From<Error> for crate::Error<D>
where
    D: Device,
{
    fn from(value: Error) -> Self {
        Self::Region(value)
    }
}

/// Specification of functionality to describe regional characteristics.
pub trait Region {
    /// Get the number of default uplink or downlink channels for the region.
    fn default_channels(is_uplink: bool) -> usize;
    /// For a dynamic channel plan, get the 800 or 900 channel list for the region.
    fn channel_from_list(channel_id: usize) -> Result<DynamicChannel, Error>;
    /// Get the default uplink or downlink frequency based on channel index for the region.
    fn mandatory_frequency(index: usize, is_uplink: bool) -> u32;
    /// Get the default uplink data rate based on channel index for the region.
    fn mandatory_ul_data_rate_range(index: usize) -> (DR, DR);
    /// Get the uplink data rate range.
    fn ul_data_rate_range() -> (DR, DR);
    /// Get the default data rate for the region.
    fn default_data_rate() -> DR;
    /// Override the uplink data rate based on region, frame type (join or data), and frequency.
    fn override_ul_data_rate_if_necessary(dr: DR, frame: Frame, ul_frequency: u32) -> DR;
    /// Get the default coding rate for the region.
    fn default_coding_rate() -> CodingRate;
    /// Get the default RX2 frequency for the region.
    fn default_rx2_frequency() -> u32;
    /// Get the default RX2 data rate for the region.
    fn default_rx2_data_rate() -> DR;
    /// Get the maximum EIRP for the region.
    fn max_eirp() -> u8;
    /// Get the minimum frequency for the region.
    fn min_frequency() -> u32;
    /// Get the maximum frequency for the region.
    fn max_frequency() -> u32;
    /// Convert the data rate to spreading factor and bandwidth for the region.
    fn convert_data_rate(dr: DR) -> Result<Datarate, Error>;
    /// get next data rate for adaptive data rate back off
    /// return None when the next data rate would be the default
    fn next_adr_data_rate(current_dr: Option<DR>) -> Option<DR>;
    /// For the region, determine the RX1 data rate based on the uplink data rate and data rate offset.
    fn get_rx1_dr(ul_dr: DR, rx1_dr_offset: u8) -> Result<DR, Error>;
    /// Does the region support TXParamSetupReq packet processing?
    fn supports_tx_param_setup() -> bool;
    /// Based on the LinkADRReq packet and the region, modify the configured transmission power.
    fn modify_dbm(tx_power: u8, cur_dbm: Option<u8>, max_eirp: u8) -> Result<Option<u8>, Error>;
    /// Get the default RX delay for the region.
    fn default_rx_delay() -> u16 {
        1000
    }
    /// Get the default RX1 data rate offset for the region.
    fn default_rx1_data_rate_offset() -> u8;
    /// Get the default delay from now before opening the RX1 window for a join accept packet from a network server.
    fn default_join_accept_delay1() -> u16 {
        5000
    }
    /// Get the default delay from now before opening the RX2 window for a join accept packet from a network server.
    fn default_join_accept_delay2() -> u16 {
        Self::default_join_accept_delay1() + 1000
    }
    /// Get the default ADR acknowledgement limit for the region.
    fn default_adr_ack_limit() -> u8 {
        64
    }
    /// Get the default ADR acknowledgement delay for the region.
    fn default_adr_ack_delay() -> u8 {
        32
    }
}

pub mod eu868;
pub mod us915;
