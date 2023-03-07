use core::marker::PhantomData;

use crate::{
    device::radio::types::{CodingRate, RfConfig},
    Frame, DR,
};

const JOIN_CHANNELS: [u32; 3] = [868_100_000, 868_300_000, 868_500_000];
mod datarates;
use datarates::*;

use super::channel_plan::ChannelPlan;

pub struct Eu868<C>
where
    C: ChannelPlan,
{
    channel_plan: PhantomData<C>,
}
impl<C> crate::mac::Region for Eu868<C>
where
    C: ChannelPlan,
{
    fn default_channels() -> u8 {
        3
    }
    fn min_frequency() -> u32 {
        863000000
    }
    fn max_frequency() -> u32 {
        870000000
    }

    fn min_data_rate() -> DR {
        DR::_0
    }

    fn max_data_rate() -> DR {
        DR::_7
    }
    fn default_data_rate() -> DR {
        DR::_0
    }
    fn default_coding_rate() -> CodingRate {
        CodingRate::_4_5
    }
    fn max_eirp() -> i8 {
        14
    }
    fn supports_tx_param_setup() -> bool {
        false
    }
    fn create_rf_config(frame: Frame, random: u32, data_rate: Option<DR>) -> RfConfig {
        match frame {
            Frame::Join => {
                let channel = random as usize % JOIN_CHANNELS.len();
                let data_rate = &DATARATES[data_rate.unwrap_or(Self::default_data_rate()) as usize];
                RfConfig {
                    frequency: JOIN_CHANNELS[channel],
                    bandwidth: data_rate.bandwidth.clone(),
                    spreading_factor: data_rate.spreading_factor.clone(),
                    coding_rate: Self::default_coding_rate(),
                }
            }
            Frame::Data => {
                todo!()
            }
        }
    }

    fn modify_dbm(tx_power: u8, cur_dbm: Option<i8>, max_eirp: i8) -> Result<Option<i8>, ()> {
        match tx_power {
            0..=7 => Ok(Some(max_eirp - (tx_power * 2) as i8)),
            15 => Ok(cur_dbm),
            _ => Err(()),
        }
    }
}
