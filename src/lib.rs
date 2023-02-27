#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(concat_idents)]

use channel_mask::ChannelMask;
use frequency::Frequency;

mod channel_mask;
mod encoding;
mod frequency;
mod mac;
mod radio;
mod timer;
enum Error<PhyError> {
    Radio(PhyError),
    InvalidMic,
    InvalidDevAddr,
    UnableToDecodePayload(&'static str),
    NetworkNotJoined,
    SessionExpired,
    FOptsFull,
    UnableToPreparePayload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
pub enum CfList {
    DynamicChannel([Frequency; 5]),
    FixedChannel([ChannelMask; 4]),
}
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
pub(crate) enum Frame {
    Join,
    Data,
}
pub(crate) enum Window {
    _1,
    _2,
}
