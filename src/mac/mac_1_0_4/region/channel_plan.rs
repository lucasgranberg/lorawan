use core::marker::PhantomData;

use heapless::Vec;

use crate::{
    channel_mask::ChannelMask,
    encoding::maccommands::{DlChannelReqPayload, NewChannelReqPayload},
    frequency::Frequency,
    DR,
};

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
    fn get_random_channel(&self, random: u32, data_rate: DR) -> Result<Self::Channel, Error>;
    fn handle_new_channel_req(&mut self, payload: NewChannelReqPayload) -> Result<(), Error>;
    fn check_uplink_frequency_exists(&self, index: usize) -> bool;
    fn handle_channel_mask(
        &mut self,
        new_mask: &mut [bool; 80],
        channel_mask: ChannelMask,
        channel_mask_ctrl: u8,
    ) -> Result<(), ()>;
    fn get_channel_mask(&self) -> [bool; 80];
    fn set_channel_mask(&mut self, mask: [bool; 80]) -> Result<(), Error>;
    fn handle_dl_channel_req(&mut self, payload: DlChannelReqPayload) -> Result<(), Error>;
}
#[derive(Debug, Clone, Copy)]
pub struct DynamicChannel {
    pub(crate) frequency: Frequency,
    pub(crate) dl_frequency: Option<Frequency>,
    pub(crate) max_data_rate: u8,
    pub(crate) min_data_rate: u8,
}
impl Channel for DynamicChannel {
    fn get_frequency(&self) -> Frequency {
        self.frequency
    }
}
pub struct DynamicChannelPlan<R, const N: usize = 16>
where
    R: Region,
{
    channels: [Option<DynamicChannel>; N],
    mask: [bool; 80],
    region: PhantomData<R>,
}

impl<R, const N: usize> Default for DynamicChannelPlan<R, N>
where
    R: Region,
{
    fn default() -> Self {
        let mut channels = [None; N];
        for (index, frequency) in R::mandatory_frequencies().iter().enumerate() {
            channels[index] = Some(DynamicChannel {
                frequency: Frequency::new_from_raw(&frequency.to_le_bytes()),
                dl_frequency: None,
                max_data_rate: R::max_data_rate_join_req() as u8,
                min_data_rate: R::min_data_rate_join_req() as u8,
            })
        }
        Self {
            channels,
            mask: [true; 80],
            region: Default::default(),
        }
    }
}
impl<R, const N: usize> ChannelPlan<R> for DynamicChannelPlan<R, N>
where
    R: Region,
{
    type Channel = DynamicChannel;
    fn handle_channel_mask(
        &mut self,
        new_mask: &mut [bool; 80],
        channel_mask: ChannelMask,
        channel_mask_ctrl: u8,
    ) -> Result<(), ()> {
        match channel_mask_ctrl {
            0..=4 => {
                for i in 0..15 {
                    let index = i + (channel_mask_ctrl * 16) as usize;
                    new_mask[index] = channel_mask.is_enabled(i).unwrap()
                }
                Ok(())
            }
            5 => {
                for i in 0..9 {
                    let index = i + (channel_mask_ctrl * 16) as usize;
                    new_mask[index] = channel_mask.is_enabled(i).unwrap()
                }
                Ok(())
            }
            6 => {
                new_mask.fill(true);
                Ok(())
            }
            _ => Err(()),
        }
    }
    fn get_mut_channel(&mut self, index: usize) -> Option<&mut Option<DynamicChannel>> {
        self.channels.get_mut(index)
    }

    fn get_random_channel(&self, random: u32, data_rate: DR) -> Result<DynamicChannel, Error> {
        let mut valid_channels: Vec<&DynamicChannel, N> = Vec::new();
        for valid_channel in self
            .channels
            .iter()
            .enumerate()
            .filter_map(|(index, c)| match c {
                Some(ch)
                    if (ch.min_data_rate..ch.max_data_rate).contains(&(data_rate as u8))
                        && self.mask[index] =>
                {
                    Some(ch)
                }
                _ => None,
            })
        {
            valid_channels.push(valid_channel).unwrap();
        }
        if valid_channels.is_empty() {
            Err(Error::NoValidChannelFound)
        } else {
            Ok(*&(*valid_channels
                .get((random % valid_channels.len() as u32) as usize)
                .unwrap())
            .clone())
        }
    }

    fn handle_new_channel_req(&mut self, payload: NewChannelReqPayload) -> Result<(), Error> {
        if (payload.channel_index() as usize) < self.channels.len() {
            self.channels[payload.channel_index() as usize] = Some(DynamicChannel {
                frequency: payload.frequency(),
                max_data_rate: payload.data_rate_range().max_data_rate(),
                min_data_rate: payload.data_rate_range().min_data_range(),
                dl_frequency: None,
            });
            Ok(())
        } else {
            Err(Error::InvalidChannelIndex)
        }
    }

    fn handle_dl_channel_req(&mut self, payload: DlChannelReqPayload) -> Result<(), Error> {
        let index = payload.channel_index() as usize;
        if (index) < N {
            if let Some(mut channel) = self.channels[index] {
                channel.dl_frequency = Some(payload.frequency());
                return Ok(());
            }
        }
        Err(Error::InvalidChannelIndex)
    }

    fn get_channel_mask(&self) -> [bool; 80] {
        self.mask
    }

    fn set_channel_mask(&mut self, mask: [bool; 80]) -> Result<(), Error> {
        self.mask = mask;
        Ok(())
    }

    fn check_uplink_frequency_exists(&self, index: usize) -> bool {
        if (index) < N {
            return self.channels[index].is_some();
        }
        false
    }
}
