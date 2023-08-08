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

/// Specification of basic functionality to get channel properties.
pub trait Channel {
    /// Get the uplink frequency.
    fn get_ul_frequency(&self) -> u32;
    /// Get the downlink frequency.
    fn get_dl_frequency(&self) -> u32;
    /// Get the uplink data rate range.
    fn get_ul_data_rate_range(&self) -> (DR, DR);
}

/// Specification of functionality to handle a channel plan for a region.
pub trait ChannelPlan<R>
where
    R: Region,
{
    /// Dynamic or fixed channel type.
    type Channel: Channel;

    /// Get an active channel randomly from each channel block based on the frame type (join or data).  The resulting
    /// collection may be sparesely populated.
    fn get_random_channels_from_blocks(
        &self,
        channel_block_randoms: [u32; NUM_OF_CHANNEL_BLOCKS],
        frame: Frame,
    ) -> Result<[Option<Self::Channel>; NUM_OF_CHANNEL_BLOCKS], Error>;
    /// Handle a new channel request from a network server.
    fn handle_new_channel_req(&mut self, payload: NewChannelReqPayload) -> Result<(), Error>;
    /// Does a channel exist for the given channel ID?
    fn check_uplink_frequency_exists(&self, index: usize) -> bool;
    /// Create a new channel mask collection based on guidance in the input channel_mask_ctrl and input channel_mask,
    /// both contained in a LinkADRReq packet from a network server to the end device.
    fn handle_channel_mask(
        &mut self,
        new_mask: &mut [bool; MAX_CHANNELS],
        channel_mask: ChannelMask<2>,
        channel_mask_ctrl: u8,
    ) -> Result<(), Error>;
    /// Get the current channel mask collection.
    fn get_channel_mask(&self) -> [bool; MAX_CHANNELS];
    /// Set the current channel maks collection.
    fn set_channel_mask(&mut self, mask: [bool; MAX_CHANNELS]) -> Result<(), Error>;
    /// Handle a DlChannelReq packet from a network server to the end device.
    fn handle_dl_channel_req(&mut self, payload: DlChannelReqPayload) -> Result<(), Error>;
    /// Handle the CFList included in a JoinAccept packet from a network server to the end device.
    fn handle_cf_list(&mut self, cf_list: CfList) -> Result<(), Error>;
    /// Does the uplink frequency exist in the channel plan?
    fn validate_frequency(&self, frequency: u32) -> Result<(), Error>;
    /// Reactivate channels for ADR
    fn reactivate_channels(&mut self);
}
