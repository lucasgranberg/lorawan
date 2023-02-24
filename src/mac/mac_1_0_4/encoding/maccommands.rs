// Copyright (c) 2018,2020 Ivaylo Petrov
//
// Licensed under the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//
// author: Ivaylo Petrov <ivajloip@gmail.com>

use crate::encoding::maccommands::*;

/// MacCommand represents the enumeration of all LoRaWAN MACCommands.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq)]
pub enum UplinkMacCommand<'a> {
    LinkCheckReq(LinkCheckReqPayload),
    LinkADRAns(LinkADRAnsPayload<'a>),
    DutyCycleAns(DutyCycleAnsPayload),
    RXParamSetupAns(RXParamSetupAnsPayload<'a>),
    DevStatusAns(DevStatusAnsPayload<'a>),
    NewChannelAns(NewChannelAnsPayload<'a>),
    RXTimingSetupAns(RXTimingSetupAnsPayload),
    TXParamSetupAns(TXParamSetupAnsPayload),
    DlChannelAns(DlChannelAnsPayload),
    DeviceTimeReq(DeviceTimeReqPayload),
}
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq)]
pub enum DownlinkMacCommand<'a> {
    LinkCheckAns(LinkCheckAnsPayload<'a>),
    LinkADRReq(LinkADRReqPayload<'a>),
    DutyCycleReq(DutyCycleReqPayload<'a>),
    RXParamSetupReq(RXParamSetupReqPayload<'a>),
    DevStatusReq(DevStatusReqPayload),
    NewChannelReq(NewChannelReqPayload<'a>),
    RXTimingSetupReq(RXTimingSetupReqPayload<'a>),
    TXParamSetupReq(TXParamSetupReqPayload<'a>),
    DlChannelReq(DlChannelReqPayload<'a>),
    DeviceTimeAns(DeviceTimeAnsPayload<'a>),
}

impl<'a> UplinkMacCommand<'a> {
    #![allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        match *self {
            Self::LinkCheckReq(_) => LinkCheckReqPayload::len(),
            Self::LinkADRAns(_) => LinkADRAnsPayload::len(),
            Self::DutyCycleAns(_) => DutyCycleAnsPayload::len(),
            Self::RXParamSetupAns(_) => RXParamSetupAnsPayload::len(),
            Self::DevStatusAns(_) => DevStatusAnsPayload::len(),
            Self::NewChannelAns(_) => NewChannelAnsPayload::len(),
            Self::RXTimingSetupAns(_) => RXTimingSetupAnsPayload::len(),
            Self::TXParamSetupAns(_) => TXParamSetupAnsPayload::len(),
            Self::DlChannelAns(_) => DlChannelAnsPayload::len(),
            Self::DeviceTimeReq(_) => DeviceTimeReqPayload::len(),
        }
    }
    pub fn uplink(&self) -> bool {
        match *self {
            Self::LinkCheckReq(_) => LinkCheckReqPayload::uplink(),
            Self::LinkADRAns(_) => LinkADRAnsPayload::uplink(),
            Self::DutyCycleAns(_) => DutyCycleAnsPayload::uplink(),
            Self::RXParamSetupAns(_) => RXParamSetupAnsPayload::uplink(),
            Self::DevStatusAns(_) => DevStatusAnsPayload::uplink(),
            Self::NewChannelAns(_) => NewChannelAnsPayload::uplink(),
            Self::RXTimingSetupAns(_) => RXTimingSetupAnsPayload::uplink(),
            Self::TXParamSetupAns(_) => TXParamSetupAnsPayload::uplink(),
            Self::DlChannelAns(_) => DlChannelAnsPayload::uplink(),
            Self::DeviceTimeReq(_) => DeviceTimeReqPayload::uplink(),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        match *self {
            Self::LinkCheckReq(_) => &[],
            Self::LinkADRAns(ref v) => v.0,
            Self::DutyCycleAns(_) => &[],
            Self::RXParamSetupAns(ref v) => v.0,
            Self::DevStatusAns(ref v) => v.0,
            Self::NewChannelAns(ref v) => v.0,
            Self::RXTimingSetupAns(_) => &[],
            Self::TXParamSetupAns(_) => &[],
            Self::DlChannelAns(_) => &[],
            Self::DeviceTimeReq(_) => &[],
        }
    }
}
impl<'a> DownlinkMacCommand<'a> {
    #![allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        match *self {
            Self::LinkCheckAns(_) => LinkCheckAnsPayload::len(),
            Self::LinkADRReq(_) => LinkADRReqPayload::len(),
            Self::DutyCycleReq(_) => DutyCycleReqPayload::len(),
            Self::RXParamSetupReq(_) => RXParamSetupReqPayload::len(),
            Self::DevStatusReq(_) => DevStatusReqPayload::len(),
            Self::NewChannelReq(_) => NewChannelReqPayload::len(),
            Self::RXTimingSetupReq(_) => RXTimingSetupReqPayload::len(),
            Self::TXParamSetupReq(_) => TXParamSetupReqPayload::len(),
            Self::DlChannelReq(_) => DlChannelReqPayload::len(),
            Self::DeviceTimeAns(_) => DeviceTimeAnsPayload::len(),
        }
    }
    pub fn uplink(&self) -> bool {
        match *self {
            Self::LinkCheckAns(_) => LinkCheckAnsPayload::uplink(),
            Self::LinkADRReq(_) => LinkADRReqPayload::uplink(),
            Self::DutyCycleReq(_) => DutyCycleReqPayload::uplink(),
            Self::RXParamSetupReq(_) => RXParamSetupReqPayload::uplink(),
            Self::DevStatusReq(_) => DevStatusReqPayload::uplink(),
            Self::NewChannelReq(_) => NewChannelReqPayload::uplink(),
            Self::RXTimingSetupReq(_) => RXTimingSetupReqPayload::uplink(),
            Self::TXParamSetupReq(_) => TXParamSetupReqPayload::uplink(),
            Self::DlChannelReq(_) => DlChannelReqPayload::uplink(),
            Self::DeviceTimeAns(_) => DeviceTimeAnsPayload::uplink(),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        match *self {
            Self::LinkCheckAns(ref v) => v.0,
            Self::LinkADRReq(ref v) => v.0,
            Self::DutyCycleReq(ref v) => v.0,
            Self::RXParamSetupReq(ref v) => v.0,
            Self::DevStatusReq(_) => &[],
            Self::NewChannelReq(ref v) => v.0,
            Self::RXTimingSetupReq(ref v) => v.0,
            Self::TXParamSetupReq(ref v) => v.0,
            Self::DlChannelReq(ref v) => v.0,
            Self::DeviceTimeAns(ref v) => v.0,
        }
    }
}

impl<'a> SerializableMacCommand for DownlinkMacCommand<'a> {
    fn payload_bytes(&self) -> &[u8] {
        self.bytes()
    }

    fn cid(&self) -> u8 {
        match *self {
            Self::LinkCheckAns(_) => LinkCheckAnsPayload::cid(),
            Self::LinkADRReq(_) => LinkADRReqPayload::cid(),
            Self::DutyCycleReq(_) => DutyCycleReqPayload::cid(),
            Self::RXParamSetupReq(_) => RXParamSetupReqPayload::cid(),
            Self::DevStatusReq(_) => DevStatusReqPayload::cid(),
            Self::NewChannelReq(_) => NewChannelReqPayload::cid(),
            Self::RXTimingSetupReq(_) => RXTimingSetupReqPayload::cid(),
            Self::TXParamSetupReq(_) => TXParamSetupReqPayload::cid(),
            Self::DlChannelReq(_) => DlChannelReqPayload::cid(),
            Self::DeviceTimeAns(_) => DeviceTimeAnsPayload::cid(),
        }
    }

    fn payload_len(&self) -> usize {
        self.len()
    }
}

impl<'a> SerializableMacCommand for UplinkMacCommand<'a> {
    fn payload_bytes(&self) -> &[u8] {
        self.bytes()
    }

    fn cid(&self) -> u8 {
        match *self {
            Self::LinkCheckReq(_) => LinkCheckReqPayload::cid(),
            Self::LinkADRAns(_) => LinkADRAnsPayload::cid(),
            Self::DutyCycleAns(_) => DutyCycleAnsPayload::cid(),
            Self::RXParamSetupAns(_) => RXParamSetupAnsPayload::cid(),
            Self::DevStatusAns(_) => DevStatusAnsPayload::cid(),
            Self::NewChannelAns(_) => NewChannelAnsPayload::cid(),
            Self::RXTimingSetupAns(_) => RXTimingSetupAnsPayload::cid(),
            Self::TXParamSetupAns(_) => TXParamSetupAnsPayload::cid(),
            Self::DlChannelAns(_) => DlChannelAnsPayload::cid(),
            Self::DeviceTimeReq(_) => DeviceTimeReqPayload::cid(),
        }
    }

    fn payload_len(&self) -> usize {
        self.len()
    }
}
