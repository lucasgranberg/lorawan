use super::Error;
use crate::{
    device::radio::types::{Bandwidth, CodingRate, Datarate, SpreadingFactor},
    DR,
};

const JOIN_CHANNELS: [u32; 3] = [868_100_000, 868_300_000, 868_500_000];

pub struct Eu868 {}
impl crate::mac::Region for Eu868 {
    fn mandatory_frequencies() -> &'static [u32] {
        &JOIN_CHANNELS
    }
    fn default_channels() -> u8 {
        3
    }
    fn min_frequency() -> u32 {
        863000000
    }
    fn max_frequency() -> u32 {
        870000000
    }
    fn default_rx2_frequency() -> u32 {
        869525000
    }

    fn min_data_rate() -> DR {
        DR::_0
    }

    fn max_data_rate() -> DR {
        DR::_7
    }
    fn min_data_rate_join_req() -> DR {
        DR::_0
    }

    fn max_data_rate_join_req() -> DR {
        DR::_5
    }
    fn default_rx2_data_rate() -> DR {
        DR::_0
    }
    fn default_data_rate() -> DR {
        DR::_0
    }

    fn default_rx1_data_rate_offset() -> DR {
        DR::_0
    }

    fn convert_data_rate(dr: DR) -> Result<Datarate, super::Error> {
        match dr {
            DR::_0 => Ok(Datarate {
                spreading_factor: SpreadingFactor::_12,
                bandwidth: Bandwidth::_125KHz,
            }),
            DR::_1 => Ok(Datarate {
                spreading_factor: SpreadingFactor::_11,
                bandwidth: Bandwidth::_125KHz,
            }),
            DR::_2 => Ok(Datarate {
                spreading_factor: SpreadingFactor::_10,
                bandwidth: Bandwidth::_125KHz,
            }),
            DR::_3 => Ok(Datarate {
                spreading_factor: SpreadingFactor::_9,
                bandwidth: Bandwidth::_125KHz,
            }),
            DR::_4 => Ok(Datarate {
                spreading_factor: SpreadingFactor::_8,
                bandwidth: Bandwidth::_125KHz,
            }),
            DR::_5 => Ok(Datarate {
                spreading_factor: SpreadingFactor::_7,
                bandwidth: Bandwidth::_125KHz,
            }),
            DR::_6 => Ok(Datarate {
                spreading_factor: SpreadingFactor::_7,
                bandwidth: Bandwidth::_250KHz,
            }),
            _ => Err(super::Error::DataRateNotSupported),
        }
    }
    fn default_coding_rate() -> CodingRate {
        CodingRate::_4_5
    }
    fn max_eirp() -> i8 {
        16
    }
    fn supports_tx_param_setup() -> bool {
        false
    }

    fn modify_dbm(tx_power: u8, cur_dbm: Option<i8>, max_eirp: i8) -> Result<Option<i8>, Error> {
        match tx_power {
            0..=7 => Ok(Some(max_eirp - (tx_power * 2) as i8)),
            15 => Ok(cur_dbm),
            _ => Err(Error::InvalidTxPower),
        }
    }

    fn get_receive_window(rx_dr_offset: DR, downstream_dr: DR) -> DR {
        let downstream_dr_nr = downstream_dr as u8;
        let rx_dr_offset_nr = rx_dr_offset as u8;
        match rx_dr_offset as u8 {
            1..=7 if rx_dr_offset_nr < downstream_dr_nr => {
                (downstream_dr_nr + rx_dr_offset_nr).try_into().unwrap()
            }
            8 | 10 if downstream_dr == DR::_0 => DR::_1,
            9 | 11 if downstream_dr_nr < 2 => (downstream_dr_nr + 1).try_into().unwrap(),
            _ => DR::_0,
        }
    }
}
