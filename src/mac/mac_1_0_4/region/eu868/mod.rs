use crate::{device::radio::types::RfConfig, Frame, DR};

const JOIN_CHANNELS: [u32; 3] = [868_100_000, 868_300_000, 868_500_000];
mod datarates;
use datarates::*;

pub struct Eu868 {}
impl crate::mac::Region for Eu868 {
    fn create_rf_config(frame: Frame, random: u32, data_rate: Option<DR>) -> RfConfig {
        match frame {
            Frame::Join => {
                let channel = random as usize % JOIN_CHANNELS.len();
                let data_rate = &DATARATES[data_rate.unwrap_or(Self::default_datarate()) as usize];
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
}
impl super::Region for Eu868 {}
