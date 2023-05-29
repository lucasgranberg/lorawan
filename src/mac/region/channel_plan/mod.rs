use crate::encoding::maccommands::{
    ChannelMask, DlChannelReqPayload, Frequency, NewChannelReqPayload,
};
use crate::encoding::parser::CfList;
use crate::mac::types::*;
pub mod dynamic;

use super::{Error, Region};

pub trait Channel {
    fn get_frequency(&self) -> Frequency;
}
pub trait ChannelPlan<R>
where
    R: Region,
{
    type Channel: Channel;
    fn get_mut_channel(&mut self, index: usize) -> Option<&mut Option<Self::Channel>>;
    fn get_random_channel(
        &self,
        random: u32,
        frame: Frame,
        data_rate: DR,
    ) -> Result<Self::Channel, Error>;
    fn handle_new_channel_req(&mut self, payload: NewChannelReqPayload) -> Result<(), Error>;
    fn check_uplink_frequency_exists(&self, index: usize) -> bool;
    fn handle_channel_mask(
        &mut self,
        new_mask: &mut [bool; 80],
        channel_mask: ChannelMask<2>,
        channel_mask_ctrl: u8,
    ) -> Result<(), Error>;
    fn get_channel_mask(&self) -> [bool; 80];
    fn set_channel_mask(&mut self, mask: [bool; 80]) -> Result<(), Error>;
    fn handle_dl_channel_req(&mut self, payload: DlChannelReqPayload) -> Result<(), Error>;
    fn handle_cf_list(&mut self, cf_list: CfList) -> Result<(), Error>;
    fn validate_frequency(&self, frequency: u32) -> Result<(), Error>;
}
