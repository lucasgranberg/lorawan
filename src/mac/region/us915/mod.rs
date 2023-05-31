use super::Error;
use crate::device::radio::types::{Bandwidth, CodingRate, Datarate, SpreadingFactor};
use crate::mac::types::DR;

pub struct Us915;

impl crate::mac::Region for Us915 {
    fn default_channels() -> u8 {
        72
    }

    fn mandatory_frequencies() -> &'static [u32] {
        todo!()
    }

    fn min_data_rate_join_req() -> DR {
        todo!()
    }

    fn max_data_rate_join_req() -> DR {
        todo!()
    }

    fn min_data_rate() -> DR {
        todo!()
    }

    fn max_data_rate() -> DR {
        todo!()
    }

    fn default_data_rate() -> DR {
        todo!()
    }

    fn default_coding_rate() -> CodingRate {
        CodingRate::_4_5
    }

    fn default_rx2_frequency() -> u32 {
        923300000
    }

    fn default_rx2_data_rate() -> DR {
        DR::_8
    }

    fn max_eirp() -> u8 {
        30
    }

    fn min_frequency() -> u32 {
        902000000
    }

    fn max_frequency() -> u32 {
        928000000
    }

    fn convert_data_rate(dr: DR) -> Result<Datarate, super::Error> {
        match dr {
            DR::_0 => Ok(Datarate {
                spreading_factor: SpreadingFactor::_10,
                bandwidth: Bandwidth::_125KHz,
            }),
            DR::_1 => Ok(Datarate {
                spreading_factor: SpreadingFactor::_9,
                bandwidth: Bandwidth::_125KHz,
            }),
            DR::_2 => Ok(Datarate {
                spreading_factor: SpreadingFactor::_8,
                bandwidth: Bandwidth::_125KHz,
            }),
            DR::_3 => Ok(Datarate {
                spreading_factor: SpreadingFactor::_7,
                bandwidth: Bandwidth::_125KHz,
            }),
            DR::_4 => Ok(Datarate {
                spreading_factor: SpreadingFactor::_8,
                bandwidth: Bandwidth::_500KHz,
            }),
            DR::_8 => Ok(Datarate {
                spreading_factor: SpreadingFactor::_12,
                bandwidth: Bandwidth::_500KHz,
            }),
            DR::_9 => Ok(Datarate {
                spreading_factor: SpreadingFactor::_11,
                bandwidth: Bandwidth::_500KHz,
            }),
            DR::_10 => Ok(Datarate {
                spreading_factor: SpreadingFactor::_10,
                bandwidth: Bandwidth::_500KHz,
            }),
            DR::_11 => Ok(Datarate {
                spreading_factor: SpreadingFactor::_9,
                bandwidth: Bandwidth::_500KHz,
            }),
            DR::_12 => Ok(Datarate {
                spreading_factor: SpreadingFactor::_8,
                bandwidth: Bandwidth::_500KHz,
            }),
            DR::_13 => Ok(Datarate {
                spreading_factor: SpreadingFactor::_7,
                bandwidth: Bandwidth::_500KHz,
            }),
            _ => Err(super::Error::DataRateNotSupported(dr)),
        }
    }

    fn get_receive_window(rx_dr_offset: DR, downstream_dr: DR) -> DR {
        let rx_dr_offset_nr = rx_dr_offset as u8;
        let start: u8 = match downstream_dr {
            DR::_0 => 10,
            DR::_1 => 11,
            DR::_2 => 12,
            DR::_3 => 13,
            DR::_4 => 14,
            DR::_5 => 10,
            DR::_6 => 11,
            _ => 10,
        };
        let nr: u8 = start - rx_dr_offset_nr;
        if nr < 8 {
            DR::_8
        } else if nr < 14 {
            nr.try_into().unwrap()
        } else {
            DR::_13
        }
    }

    fn supports_tx_param_setup() -> bool {
        false
    }

    fn modify_dbm(tx_power: u8, cur_dbm: Option<u8>, _max_eirp: u8) -> Result<Option<u8>, Error> {
        match tx_power {
            0 => Ok(Some(Self::max_eirp() - (tx_power * 2))),
            15 => Ok(cur_dbm),
            _ => Err(Error::InvalidTxPower),
        }
    }

    fn default_rx1_data_rate_offset() -> u8 {
        0
    }
}
