#![feature(type_alias_impl_trait)]
#![feature(concat_idents)]

use core::fmt::Debug;

use channel_mask::ChannelMask;
use device::Device;
use frequency::Frequency;
use mac::mac_1_0_4::region;

pub mod channel_mask;
pub mod device;
pub mod encoding;
pub mod frequency;
pub mod mac;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<D>
where
    D: Device,
{
    Device(device::Error<D>),
    Region(region::Error),
    Mac(mac::Error),
    Encoding(encoding::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DR {
    _0 = 0,
    _1 = 1,
    _2 = 2,
    _3 = 3,
    _4 = 4,
    _5 = 5,
    _6 = 6,
    _7 = 7,
    _8 = 8,
    _9 = 9,
    _10 = 10,
    _11 = 11,
    _12 = 12,
    _13 = 13,
    _14 = 14,
    _15 = 15,
}

impl TryFrom<u8> for DR {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(DR::_0),
            1 => Ok(DR::_1),
            2 => Ok(DR::_2),
            3 => Ok(DR::_3),
            4 => Ok(DR::_4),
            5 => Ok(DR::_5),
            6 => Ok(DR::_6),
            7 => Ok(DR::_7),
            8 => Ok(DR::_8),
            9 => Ok(DR::_9),
            10 => Ok(DR::_10),
            11 => Ok(DR::_11),
            12 => Ok(DR::_12),
            13 => Ok(DR::_13),
            14 => Ok(DR::_14),
            15 => Ok(DR::_15),
            _ => Err(()),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CfList {
    DynamicChannel([Frequency; 5]),
    FixedChannel([ChannelMask; 4]),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MType {
    JoinRequest,
    JoinAccept,
    UnconfirmedDataUp,
    UnconfirmedDataDown,
    ConfirmedDataUp,
    ConfirmedDataDown,
    RFU,
    Proprietary,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frame {
    Join,
    Data,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Window {
    _1,
    _2,
}
