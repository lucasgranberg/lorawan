use crate::{
    device::radio::types::{CodingRate, Datarate},
    DR,
};

pub mod channel_plan;

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
    fn max_eirp() -> i8;
    fn min_frequency() -> u32;
    fn max_frequency() -> u32;
    fn convert_data_rate(dr: DR) -> Option<Datarate>;
    fn get_receive_window(rx_dr_offset: DR, downstream_dr: DR) -> DR;
    fn supports_tx_param_setup() -> bool;
    fn modify_dbm(tx_power: u8, cur_dbm: Option<i8>, max_eirp: i8) -> Result<Option<i8>, ()>;

    fn default_receive_delay1() -> u32 {
        1000
    }
    fn default_receive_delay2() -> u32 {
        Self::default_receive_delay1() + 1
    }
    fn default_rx1_data_rate_offset() -> DR {
        DR::_0
    }
    fn default_join_accept_delay1() -> u32 {
        5000
    }
    fn default_join_accept_delay2() -> u32 {
        Self::default_join_accept_delay1() + 1
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
