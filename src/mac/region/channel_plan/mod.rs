//! Specification of the functionality implemented for dynamic and fixed channel plans.

use crate::encoding::maccommands::{ChannelMask, DlChannelReqPayload, NewChannelReqPayload};
use crate::encoding::parser::CfList;
use crate::mac::types::*;
pub mod dynamic;
pub mod fixed;

use super::{Error, Region};

/// Maximum number of channels in a deployed channel plan.
pub const MAX_CHANNELS: usize = 80;
/// Maximum number of configurable channels in an 800 fixed list.
pub const MAX_800_CHANNELS: usize = 40;
/// Maximum number of configurable channels in a 900 fixed list.
pub const MAX_900_CHANNELS: usize = 96;
/// Number of channel blocks for a channel plan.
pub const NUM_OF_CHANNEL_BLOCKS: usize = 10;
/// Number of channels in a channel block.
pub const NUM_OF_CHANNELS_IN_BLOCK: usize = 8;

pub trait Channel {
    fn get_ul_frequency(&self) -> u32;
    fn get_dl_frequency(&self) -> u32;
    fn get_ul_data_rate_range(&self) -> (DR, DR);
}
pub trait ChannelPlan<R>
where
    R: Region,
{
    /// Dynamic or fixed channel type.
    type Channel: Channel;

    fn get_mut_channel(&mut self, index: usize) -> Option<&mut Option<Self::Channel>>;
    fn get_random_channels_from_blocks(
        &self,
        channel_block_randoms: [u32; NUM_OF_CHANNEL_BLOCKS],
        frame: Frame,
    ) -> Result<[Option<Self::Channel>; NUM_OF_CHANNEL_BLOCKS], Error>;
    fn handle_new_channel_req(&mut self, payload: NewChannelReqPayload) -> Result<(), Error>;
    fn check_uplink_frequency_exists(&self, index: usize) -> bool;
    fn handle_channel_mask(
        &mut self,
        new_mask: &mut [bool; MAX_CHANNELS],
        channel_mask: ChannelMask<2>,
        channel_mask_ctrl: u8,
    ) -> Result<(), Error>;
    fn get_channel_mask(&self) -> [bool; MAX_CHANNELS];
    fn set_channel_mask(&mut self, mask: [bool; MAX_CHANNELS]) -> Result<(), Error>;
    fn handle_dl_channel_req(&mut self, payload: DlChannelReqPayload) -> Result<(), Error>;
    fn handle_cf_list(&mut self, cf_list: CfList) -> Result<(), Error>;
    fn validate_frequency(&self, frequency: u32) -> Result<(), Error>;
}
