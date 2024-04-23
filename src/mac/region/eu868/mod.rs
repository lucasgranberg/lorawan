//! Processing for the EU868 region, which uses a dynamic channel plan.

use lora_modulation::{Bandwidth, CodingRate, SpreadingFactor};

use super::channel_plan::dynamic::{DynamicChannel, DynamicChannelPlan};
use super::Error;
use crate::device::types::Datarate;
use crate::mac::types::{Frame, DR};

const JOIN_CHANNELS: [u32; 3] = [868_100_000, 868_300_000, 868_500_000];

/// Specific processing for the EU868 region.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EU868;
impl crate::mac::Region for EU868 {
    fn default_channels(_is_uplink: bool) -> usize {
        3
    }
    fn channel_from_list(channel_id: usize) -> Result<DynamicChannel, Error> {
        DynamicChannelPlan::<EU868>::get_800_channel(channel_id)
    }
    fn mandatory_frequency(index: usize, _is_uplink: bool) -> u32 {
        JOIN_CHANNELS[index]
    }
    fn mandatory_ul_data_rate_range(_index: usize) -> (DR, DR) {
        (DR::_0, DR::_5)
    }
    fn ul_data_rate_range() -> (DR, DR) {
        (DR::_0, DR::_5)
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
    fn default_rx2_data_rate() -> DR {
        DR::_0
    }
    fn default_data_rate() -> DR {
        DR::_0
    }
    fn override_ul_data_rate_if_necessary(dr: DR, _frame: Frame, _ul_frequency: u32) -> DR {
        if dr.in_range(EU868::ul_data_rate_range()) {
            dr
        } else {
            EU868::default_data_rate()
        }
    }

    fn default_rx1_data_rate_offset() -> u8 {
        0
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
            _ => Err(super::Error::DataRateNotSupported(dr)),
        }
    }

    fn next_adr_data_rate(current_dr: Option<DR>) -> Option<DR> {
        match current_dr {
            Some(DR::_0) => None,
            Some(DR::_1) => Some(DR::_0),
            Some(DR::_2) => Some(DR::_1),
            Some(DR::_3) => Some(DR::_2),
            Some(DR::_4) => Some(DR::_3),
            Some(DR::_5) => Some(DR::_4),
            Some(DR::_6) => Some(DR::_5),
            _ => Some(DR::_0),
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
            0..=7 => {
                let next_dbm = max_eirp.checked_sub_unsigned(tx_power * 2);
                if next_dbm.is_none() {
                    Err(Error::InvalidTxPower)
                } else {
                    Ok(next_dbm)
                }
            }
            15 => Ok(cur_dbm),
            _ => Err(Error::InvalidTxPower),
        }
    }

    fn get_rx1_dr(ul_dr: DR, rx1_dr_offset: u8) -> Result<DR, super::Error> {
        if rx1_dr_offset > 5 {
            return Err(super::Error::UnsupportedRx1DROffset(ul_dr, rx1_dr_offset));
        }
        let dl_dr_matrix = [
            [DR::_0, DR::_0, DR::_0, DR::_0, DR::_0, DR::_0],
            [DR::_1, DR::_0, DR::_0, DR::_0, DR::_0, DR::_0],
            [DR::_2, DR::_1, DR::_0, DR::_0, DR::_0, DR::_0],
            [DR::_3, DR::_2, DR::_1, DR::_0, DR::_0, DR::_0],
            [DR::_4, DR::_3, DR::_2, DR::_1, DR::_0, DR::_0],
            [DR::_5, DR::_4, DR::_3, DR::_2, DR::_1, DR::_0],
        ];
        match ul_dr {
            DR::_0 => Ok(dl_dr_matrix[0][rx1_dr_offset as usize]),
            DR::_1 => Ok(dl_dr_matrix[1][rx1_dr_offset as usize]),
            DR::_2 => Ok(dl_dr_matrix[2][rx1_dr_offset as usize]),
            DR::_3 => Ok(dl_dr_matrix[3][rx1_dr_offset as usize]),
            DR::_4 => Ok(dl_dr_matrix[4][rx1_dr_offset as usize]),
            DR::_5 => Ok(dl_dr_matrix[5][rx1_dr_offset as usize]),
            _ => Err(super::Error::UnsupportedRx1DROffset(ul_dr, rx1_dr_offset)),
        }
    }
}
