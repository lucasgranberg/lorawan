//! Dynamic channel plan processing.

use crate::encoding::maccommands::{ChannelMask, DlChannelReqPayload, NewChannelReqPayload};
use crate::encoding::parser::CfList;
use crate::mac::region::{Error, Region};
use crate::mac::types::*;
use core::marker::PhantomData;

use super::{
    Channel, ChannelPlan, MAX_900_CHANNELS, MAX_CHANNELS, NUM_OF_CHANNELS_IN_BLOCK,
    NUM_OF_CHANNEL_BLOCKS,
};

/// Composition of properties and functions needed to represent a dynamic channel.
#[derive(Debug, Clone, Copy)]
pub struct DynamicChannel {
    pub(crate) ul_frequency: u32,
    pub(crate) dl_frequency: u32,
    pub(crate) ul_data_rate_range: (DR, DR),
}
impl Channel for DynamicChannel {
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

/// Composition of properties and functions needed to control a dynamic channel plan.
pub struct DynamicChannelPlan<R>
where
    R: Region,
{
    channels: [Option<DynamicChannel>; MAX_CHANNELS],
    mask: [bool; MAX_CHANNELS],
    region: PhantomData<R>,
}

impl<R> DynamicChannelPlan<R>
where
    R: Region,
{
    /// Use a channel ID to obtain a channel from the 800 channel list.
    pub fn get_800_channel(id: usize) -> Result<DynamicChannel, Error> {
        match id {
            0..=34 => {
                let frequency = 863100000 + (200000 * id as u32);
                Ok(DynamicChannel {
                    ul_frequency: frequency,
                    dl_frequency: frequency,
                    ul_data_rate_range: R::ul_data_rate_range(),
                })
            }
            35 => Ok(DynamicChannel {
                ul_frequency: 865062500,
                dl_frequency: 865062500,
                ul_data_rate_range: R::ul_data_rate_range(),
            }),
            36 => Ok(DynamicChannel {
                ul_frequency: 865402500,
                dl_frequency: 865402500,
                ul_data_rate_range: R::ul_data_rate_range(),
            }),
            37 => Ok(DynamicChannel {
                ul_frequency: 865602500,
                dl_frequency: 865602500,
                ul_data_rate_range: R::ul_data_rate_range(),
            }),
            38 => Ok(DynamicChannel {
                ul_frequency: 86578500,
                dl_frequency: 86578500,
                ul_data_rate_range: R::ul_data_rate_range(),
            }),
            39 => Ok(DynamicChannel {
                ul_frequency: 86598500,
                dl_frequency: 86598500,
                ul_data_rate_range: R::ul_data_rate_range(),
            }),
            _ => Err(Error::InvalidChannelIndex),
        }
    }

    /// Use a channel ID to obtain a channel from the 900 channel list.
    pub fn get_900_channel(id: usize) -> Result<DynamicChannel, Error> {
        if id >= MAX_900_CHANNELS {
            return Err(Error::InvalidChannelIndex);
        }
        let frequency = 915100000 + (100000 * id as u32);
        Ok(DynamicChannel {
            ul_frequency: frequency,
            dl_frequency: frequency,
            ul_data_rate_range: R::ul_data_rate_range(),
        })
    }
}

impl<R> Default for DynamicChannelPlan<R>
where
    R: Region,
{
    fn default() -> Self {
        let mut channels = [None; MAX_CHANNELS];
        let mut mask = [false; MAX_CHANNELS];
        for index in 0..R::default_channels(true) {
            channels[index] = Some(DynamicChannel {
                ul_frequency: R::mandatory_frequency(index, true),
                dl_frequency: R::mandatory_frequency(index, false),
                ul_data_rate_range: R::mandatory_ul_data_rate_range(index),
            });
            mask[index] = true;
        }
        Self { channels, mask, region: Default::default() }
    }
}

impl<R> ChannelPlan<R> for DynamicChannelPlan<R>
where
    R: Region,
{
    type Channel = DynamicChannel;

    fn get_mut_channel(&mut self, index: usize) -> Option<&mut Option<DynamicChannel>> {
        self.channels.get_mut(index)
    }

    // Randomly choose one valid channel (if one exists) from each channel block  The returned array is likely sparsely populated.
    // The first block initially contains 3 valid join channels, one of which will be randomly chosen for a join request as the first block
    // representative.  This may need to change if more valid join channels are added to the first block.
    fn get_random_channels_from_blocks(
        &self,
        channel_block_randoms: [u32; NUM_OF_CHANNEL_BLOCKS],
        frame: Frame,
    ) -> Result<[Option<DynamicChannel>; NUM_OF_CHANNEL_BLOCKS], Error> {
        let mut random_channels: [Option<DynamicChannel>; NUM_OF_CHANNEL_BLOCKS] =
            [None; NUM_OF_CHANNEL_BLOCKS];

        for i in 0..NUM_OF_CHANNEL_BLOCKS {
            let mut count = 0usize;
            let mut available_channel_ids_in_block: [Option<usize>; NUM_OF_CHANNELS_IN_BLOCK] =
                [None; NUM_OF_CHANNELS_IN_BLOCK];

            if (i == 0) && (frame == Frame::Join) {
                for j in 0..R::default_channels(true) {
                    available_channel_ids_in_block[count] = Some(j);
                    count += 1;
                }
            } else {
                for j in 0..NUM_OF_CHANNELS_IN_BLOCK {
                    let channel_index: usize = (i * NUM_OF_CHANNELS_IN_BLOCK) + j;
                    if let Some(_channel) = &self.channels[channel_index] {
                        if self.mask[channel_index] {
                            available_channel_ids_in_block[count] = Some(channel_index);
                            count += 1;
                        }
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

    fn handle_new_channel_req(&mut self, payload: NewChannelReqPayload) -> Result<(), Error> {
        if (payload.channel_index() as usize) < self.channels.len() {
            self.channels[payload.channel_index() as usize] = Some(DynamicChannel {
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
        channel_mask: ChannelMask<2>,
        channel_mask_ctrl: u8,
    ) -> Result<(), Error> {
        match channel_mask_ctrl {
            0..=4 => {
                for i in 0..15 {
                    let index = i + (channel_mask_ctrl * 16) as usize;
                    new_mask[index] = channel_mask.is_enabled(i).unwrap()
                }
                Ok(())
            }
            5 => {
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
                Ok(())
            }
            _ => Err(Error::InvalidChannelMaskCtrl),
        }
    }

    fn get_channel_mask(&self) -> [bool; MAX_CHANNELS] {
        self.mask
    }

    fn set_channel_mask(&mut self, mask: [bool; MAX_CHANNELS]) -> Result<(), Error> {
        self.mask = mask;
        Ok(())
    }

    fn handle_dl_channel_req(&mut self, payload: DlChannelReqPayload) -> Result<(), Error> {
        let index = payload.channel_index() as usize;
        if (index) < MAX_CHANNELS {
            if let Some(mut channel) = self.channels[index] {
                channel.dl_frequency = payload.frequency().value();
                return Ok(());
            }
        }
        Err(Error::InvalidChannelIndex)
    }

    fn handle_cf_list(&mut self, cf_list: CfList) -> Result<(), Error> {
        match cf_list {
            CfList::DynamicChannel(cf_list) => {
                let mut index: usize = R::default_channels(true);
                for frequency in cf_list.iter() {
                    if frequency.value() > 0 {
                        self.channels[index] = Some(DynamicChannel {
                            ul_frequency: frequency.value(),
                            dl_frequency: frequency.value(),
                            ul_data_rate_range: R::ul_data_rate_range(),
                        });
                        self.mask[index] = true;
                    }
                    index += 1;
                }
                for i in index..MAX_CHANNELS {
                    self.channels[i] = None;
                    self.mask[i] = false;
                }
                Ok(())
            }
            CfList::FixedChannel(channel_mask) => {
                let mut index: usize = R::default_channels(true);
                let mut channel_list_index: usize = 0;
                while index < MAX_CHANNELS {
                    if let Ok(channel) = R::channel_from_list(channel_list_index) {
                        if channel_mask.is_enabled(channel_list_index).unwrap() {
                            self.channels[index] = Some(channel);
                            self.mask[index] = true;
                            index += 1;
                        }
                        channel_list_index += 1;
                    } else {
                        break; // have reached the end of the channel list
                    }
                }
                for i in index..MAX_CHANNELS {
                    self.channels[i] = None;
                    self.mask[i] = false;
                }
                Ok(())
            }
        }
    }

    fn validate_frequency(&self, frequency: u32) -> Result<(), Error> {
        for channel in self.channels.iter().flatten() {
            if channel.get_ul_frequency() == frequency {
                return Ok(());
            }
        }
        Err(Error::InvalidFrequency)
    }

    fn reset_uplink_frequecy_channels(&mut self) {
        todo!()
    }
}
