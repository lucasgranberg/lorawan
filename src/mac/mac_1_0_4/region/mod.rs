use crate::device::Device;
use crate::{
    device::radio::types::{CodingRate, Datarate},
    DR,
};

pub mod channel_plan;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    InvalidTxPower,
    InvalidChannelIndex,
    InvalidChannelMaskCtrl,
    InvalidFrequency,
    DataRateNotSupported(DR),
    NoValidChannelFound,
    InvalidCfListType,
}
impl<D> From<Error> for super::Error<D>
where
    D: Device,
{
    fn from(value: Error) -> Self {
        Self::Region(value)
    }
}
pub trait Region {
    fn default_channels() -> u8;
    fn mandatory_frequencies() -> &'static [u32];
    fn min_data_rate_join_req() -> DR;
    fn max_data_rate_join_req() -> DR;
    fn min_data_rate() -> DR;
    fn max_data_rate() -> DR;
    fn default_data_rate() -> DR;
    fn default_coding_rate() -> CodingRate;
    fn default_rx2_frequency() -> u32;
    fn default_rx2_data_rate() -> DR;
    fn max_eirp() -> u8;
    fn min_frequency() -> u32;
    fn max_frequency() -> u32;
    fn convert_data_rate(dr: DR) -> Result<Datarate, Error>;
    fn get_receive_window(rx_dr_offset: DR, downstream_dr: DR) -> DR;
    fn supports_tx_param_setup() -> bool;
    fn modify_dbm(tx_power: u8, cur_dbm: Option<u8>, max_eirp: u8) -> Result<Option<u8>, Error>;

    fn default_rx_delay() -> u16 {
        1000
    }
    fn default_rx1_data_rate_offset() -> u8;
    fn default_join_accept_delay1() -> u16 {
        5000
    }
    fn default_join_accept_delay2() -> u16 {
        Self::default_join_accept_delay1() + 1000
    }
    fn default_max_fcnt_gap() -> u32 {
        16384
    }
    fn default_adr_ack_limit() -> usize {
        64
    }
    fn default_adr_ack_delay() -> u8 {
        32
    }
}

pub mod eu868;
