use core::marker::PhantomData;

use super::Error;
use crate::encoding::maccommands::Frequency;
use crate::mac::region::Region;
use crate::mac::types::DR;

use super::{Channel, ChannelPlan};

pub struct FixedChannel {
    pub(crate) frequency: u32,
}
impl Channel for FixedChannel {
    fn get_frequency(&self) -> u32 {
        self.frequency
    }
}
pub struct FixedChannelPlan<R, L>
where
    R: Region,
    L: FixedChannelList,
{
    region: PhantomData<R>,
    list: PhantomData<L>,
}

impl<R, L> ChannelPlan<R> for FixedChannelPlan<R, L>
where
    R: Region,
    L: FixedChannelList,
{
    type Channel = FixedChannel;

    fn get_mut_channel(&mut self, index: usize) -> Option<&mut Option<Self::Channel>> {
        todo!()
    }

    fn get_random_channel(
        &self,
        random: u32,
        frame: crate::mac::types::Frame,
        data_rate: crate::mac::types::DR,
    ) -> Result<Self::Channel, crate::mac::region::Error> {
        todo!()
    }

    fn handle_new_channel_req(
        &mut self,
        payload: crate::encoding::maccommands::NewChannelReqPayload,
    ) -> Result<(), crate::mac::region::Error> {
        todo!()
    }

    fn check_uplink_frequency_exists(&self, index: usize) -> bool {
        todo!()
    }

    fn handle_channel_mask(
        &mut self,
        new_mask: &mut [bool; 80],
        channel_mask: crate::encoding::maccommands::ChannelMask<2>,
        channel_mask_ctrl: u8,
    ) -> Result<(), crate::mac::region::Error> {
        todo!()
    }

    fn get_channel_mask(&self) -> [bool; 80] {
        todo!()
    }

    fn set_channel_mask(&mut self, mask: [bool; 80]) -> Result<(), crate::mac::region::Error> {
        todo!()
    }

    fn handle_dl_channel_req(
        &mut self,
        payload: crate::encoding::maccommands::DlChannelReqPayload,
    ) -> Result<(), crate::mac::region::Error> {
        Err(Error::CommandNotImplmentedForRegion)
    }

    fn handle_cf_list(
        &mut self,
        cf_list: crate::encoding::parser::CfList,
    ) -> Result<(), crate::mac::region::Error> {
        todo!()
    }

    fn validate_frequency(&self, frequency: u32) -> Result<(), crate::mac::region::Error> {
        todo!()
    }
}

pub trait FixedChannelList {
    fn len() -> usize;
    fn channel(id: usize) -> Result<FixedChannel, Error>;
}

pub struct FixedChannelList800;

impl FixedChannelList for FixedChannelList800 {
    fn len() -> usize {
        80
    }

    fn channel(id: usize) -> Result<FixedChannel, Error> {
        match id {
            0..=34 => Ok(FixedChannel {
                frequency: 863100000 + (200000 * id as u32),
            }),
            35 => Ok(FixedChannel {
                frequency: 865062500,
            }),
            36 => Ok(FixedChannel {
                frequency: 865402500,
            }),
            37 => Ok(FixedChannel {
                frequency: 865602500,
            }),
            38 => Ok(FixedChannel {
                frequency: 86578500,
            }),
            39 => Ok(FixedChannel {
                frequency: 86598500,
            }),
            _ => Err(Error::InvalidChannelIndex),
        }
    }
}

pub struct FixedChannelList900;

impl FixedChannelList for FixedChannelList900 {
    fn len() -> usize {
        96
    }

    fn channel(id: usize) -> Result<FixedChannel, Error> {
        if id < Self::len() {
            Ok(FixedChannel {
                frequency: 915100000 + (200000 * id as u32),
            })
        } else {
            Err(Error::InvalidChannelIndex)
        }
    }
}
