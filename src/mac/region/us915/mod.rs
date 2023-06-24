use super::Error;
use crate::device::radio::types::{Bandwidth, CodingRate, Datarate, SpreadingFactor};
use crate::mac::types::DR;

pub struct US915;

// TODO need to specify DR4 for the 8 500KHz uplink channels ???

impl crate::mac::Region for US915 {
    fn default_channels(is_uplink: bool) -> usize {
        if is_uplink {
            72
        } else {
            8
        }
    }

    fn mandatory_frequency(index: usize, is_uplink: bool) -> u32 {
        if is_uplink {
            // upstream: 64 (902.3 to 914.9 [+ by 0.2]) + 8 (903.0 to 914.2 [+ by 1.6])
            if index < 64 {
                (902_300_000 + (200_000 * index)) as u32
            } else {
                (903_000_000 + (1_600_000 * (index - 64))) as u32
            }
        } else {
            // downstream: 8 (923.3 to 927.5 [+ by 0.6])
            (923_300_000 + (600_000 * index)) as u32
        }
    }

    fn min_data_rate_join_req() -> DR {
        DR::_0
    }

    fn max_data_rate_join_req() -> DR {
        DR::_4
    }

    fn min_data_rate() -> DR {
        DR::_0
    }

    fn max_data_rate() -> DR {
        DR::_4
    }

    fn default_data_rate() -> DR {
        DR::_0
    }

    fn default_coding_rate() -> CodingRate {
        CodingRate::_4_5
    }

    fn default_rx2_frequency() -> u32 {
        923_300_000
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

    fn get_rx1_dr(ul_dr: DR, rx1_dr_offset: u8) -> Result<DR, super::Error> {
        if rx1_dr_offset > 3 {
            return Err(super::Error::UnsupportedRx1DROffset(ul_dr, rx1_dr_offset));
        }
        let dl_dr_matrix = [
            [DR::_10, DR::_9, DR::_8, DR::_8],
            [DR::_11, DR::_10, DR::_9, DR::_8],
            [DR::_12, DR::_11, DR::_10, DR::_9],
            [DR::_13, DR::_12, DR::_11, DR::_10],
            [DR::_13, DR::_13, DR::_12, DR::_11],
        ];
        match ul_dr {
            DR::_0 => Ok(dl_dr_matrix[0][rx1_dr_offset as usize]),
            DR::_1 => Ok(dl_dr_matrix[1][rx1_dr_offset as usize]),
            DR::_2 => Ok(dl_dr_matrix[2][rx1_dr_offset as usize]),
            DR::_3 => Ok(dl_dr_matrix[3][rx1_dr_offset as usize]),
            DR::_4 => Ok(dl_dr_matrix[4][rx1_dr_offset as usize]),
            _ => return Err(super::Error::UnsupportedRx1DROffset(ul_dr, rx1_dr_offset)),
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
