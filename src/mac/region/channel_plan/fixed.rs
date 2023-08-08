//! Fixed channel plan processing.

use core::marker::PhantomData;

use super::Error;
use crate::encoding::parser::CfList;
use crate::mac::region::Region;
use crate::mac::types::*;

use super::{Channel, ChannelPlan, MAX_CHANNELS, NUM_OF_CHANNELS_IN_BLOCK, NUM_OF_CHANNEL_BLOCKS};

/// Composition of properties and functions needed to represent a fixed channel.
#[derive(Debug, Clone, Copy)]
pub struct FixedChannel {
    pub(crate) ul_frequency: u32,
    pub(crate) dl_frequency: u32,
    pub(crate) ul_data_rate_range: (DR, DR),
}
impl Channel for FixedChannel {
    fn get_ul_frequency(&self) -> u32 {
        self.ul_frequency
    }

    fn get_dl_frequency(&self) -> u32 {
        self.dl_frequency
    }

    fn get_ul_data_rate_range(&self) -> (DR, DR) {
        self.ul_data_rate_range
    }
}

/// Composition of properties and functions needed to control a fixed channel plan.
pub struct FixedChannelPlan<R>
where
    R: Region,
{
    channels: [Option<FixedChannel>; MAX_CHANNELS],
    mask: [bool; MAX_CHANNELS],
    region: PhantomData<R>,
}

impl<R> Default for FixedChannelPlan<R>
where
    R: Region,
{
    fn default() -> Self {
        let mut channels = [None; MAX_CHANNELS];
        let mut mask = [false; MAX_CHANNELS];
        for index in 0..R::default_channels(true) {
            channels[index] = Some(FixedChannel {
                ul_frequency: R::mandatory_frequency(index, true),
                dl_frequency: R::mandatory_frequency(index % R::default_channels(false), false),
                ul_data_rate_range: R::mandatory_ul_data_rate_range(index),
            });
            mask[index] = true;
        }
        Self { channels, mask, region: Default::default() }
    }
}

impl<R> ChannelPlan<R> for FixedChannelPlan<R>
where
    R: Region,
{
    type Channel = FixedChannel;

    // Randomly choose one valid channel (if one exists) from each channel block  The returned array is likely sparsely populated.
    fn get_random_channels_from_blocks(
        &self,
        channel_block_randoms: [u32; NUM_OF_CHANNEL_BLOCKS],
        _frame: crate::mac::types::Frame,
    ) -> Result<[Option<FixedChannel>; NUM_OF_CHANNEL_BLOCKS], crate::mac::region::Error> {
        let mut random_channels: [Option<FixedChannel>; NUM_OF_CHANNEL_BLOCKS] =
            [None; NUM_OF_CHANNEL_BLOCKS];

        for i in 0..NUM_OF_CHANNEL_BLOCKS {
            let mut count = 0usize;
            let mut available_channel_ids_in_block: [Option<usize>; NUM_OF_CHANNELS_IN_BLOCK] =
                [None; NUM_OF_CHANNELS_IN_BLOCK];
            for j in 0..NUM_OF_CHANNELS_IN_BLOCK {
                let channel_index: usize = (i * NUM_OF_CHANNELS_IN_BLOCK) + j;
                if let Some(_channel) = &self.channels[channel_index] {
                    if self.mask[channel_index] {
                        available_channel_ids_in_block[count] = Some(channel_index);
                        count += 1;
                    }
                }
            }

            if count > 0 {
                let random = channel_block_randoms[i] % (count as u32);
                let channel_id = available_channel_ids_in_block[random as usize].unwrap();
                random_channels[i] = self.channels[channel_id];
            }
        }

        Ok(random_channels)
    }

    fn handle_new_channel_req(
        &mut self,
        payload: crate::encoding::maccommands::NewChannelReqPayload,
    ) -> Result<(), crate::mac::region::Error> {
        if (payload.channel_index() as usize) < self.channels.len() {
            self.channels[payload.channel_index() as usize] = Some(FixedChannel {
                ul_frequency: payload.frequency().value(),
                dl_frequency: payload.frequency().value(),
                ul_data_rate_range: (
                    DR::try_from(payload.data_rate_range().min_data_rate()).unwrap(),
                    DR::try_from(payload.data_rate_range().max_data_rate()).unwrap(),
                ),
            });
            Ok(())
        } else {
            Err(Error::InvalidChannelIndex)
        }
    }

    fn check_uplink_frequency_exists(&self, index: usize) -> bool {
        if (index) < MAX_CHANNELS {
            return self.channels[index].is_some();
        }
        false
    }

    fn handle_channel_mask(
        &mut self,
        new_mask: &mut [bool; MAX_CHANNELS],
        channel_mask: crate::encoding::maccommands::ChannelMask<2>,
        channel_mask_ctrl: u8,
    ) -> Result<(), crate::mac::region::Error> {
        match channel_mask_ctrl {
            0..=4 => {
                for i in 0..15 {
                    let index = i + (channel_mask_ctrl * 16) as usize;
                    new_mask[index] = channel_mask.is_enabled(i).unwrap()
                }
                Ok(())
            }
            5 => {
                // This algorithm does not implement the modification of single 500 KHz channels
                // for bits 0 through 7, since processing of bit 8 affects the entire 500 KHz channel block.
                // Until further evidence, the regional specification for fixed channels is taken to need clarification.
                let channel_mask_be: u16 =
                    channel_mask.get_index(0) as u16 | ((channel_mask.get_index(1) as u16) << 8);
                for i in 0..NUM_OF_CHANNEL_BLOCKS {
                    let is_channel_block_enabled = channel_mask_be & (1 << i) != 0;
                    for j in 0..NUM_OF_CHANNELS_IN_BLOCK {
                        new_mask[(i * NUM_OF_CHANNELS_IN_BLOCK) + j] = is_channel_block_enabled;
                    }
                }
                Ok(())
            }
            6 => {
                new_mask.fill(true);
                for i in 0..15 {
                    let index = i + 64;
                    new_mask[index] = channel_mask.is_enabled(i).unwrap()
                }
                Ok(())
            }
            7 => {
                new_mask.fill(false);
                for i in 0..15 {
                    let index = i + 64;
                    new_mask[index] = channel_mask.is_enabled(i).unwrap()
                }
                Ok(())
            }
            _ => Err(Error::InvalidChannelMaskCtrl),
        }
    }

    fn get_channel_mask(&self) -> [bool; MAX_CHANNELS] {
        self.mask
    }

    fn set_channel_mask(
        &mut self,
        mask: [bool; MAX_CHANNELS],
    ) -> Result<(), crate::mac::region::Error> {
        self.mask = mask;
        Ok(())
    }

    fn handle_dl_channel_req(
        &mut self,
        _payload: crate::encoding::maccommands::DlChannelReqPayload,
    ) -> Result<(), crate::mac::region::Error> {
        Err(Error::CommandNotImplementedForRegion)
    }

    fn handle_cf_list(
        &mut self,
        cf_list: crate::encoding::parser::CfList,
    ) -> Result<(), crate::mac::region::Error> {
        if let CfList::FixedChannel(channel_mask) = cf_list {
            for index in 0..MAX_CHANNELS {
                self.mask[index] = channel_mask.is_enabled(index).unwrap()
            }
            Ok(())
        } else {
            Err(Error::InvalidCfListType)
        }
    }

    fn validate_frequency(&self, _frequency: u32) -> Result<(), crate::mac::region::Error> {
        // Possibly the validation done here should only be to check that the frequency is within min/max limits.  It seems to
        // be mostly called to validate downlink frequencies, and some MAC commands using it are inapplicable to fixed channel plans.
        Err(Error::CommandNotImplementedForRegion) // ???
    }

    fn reactivate_channels(&mut self) {
        // Set all default channels to active.
        self.mask = [false; MAX_CHANNELS];
        for index in 0..R::default_channels(true) {
            self.mask[index] = true;
        }
    }
}
