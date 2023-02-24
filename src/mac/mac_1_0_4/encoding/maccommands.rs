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
pub enum MacCommand<'a> {
    LinkCheckReq(LinkCheckReqPayload),
    LinkCheckAns(LinkCheckAnsPayload<'a>),
    LinkADRReq(LinkADRReqPayload<'a>),
    LinkADRAns(LinkADRAnsPayload<'a>),
    DutyCycleReq(DutyCycleReqPayload<'a>),
    DutyCycleAns(DutyCycleAnsPayload),
    RXParamSetupReq(RXParamSetupReqPayload<'a>),
    RXParamSetupAns(RXParamSetupAnsPayload<'a>),
    DevStatusReq(DevStatusReqPayload),
    DevStatusAns(DevStatusAnsPayload<'a>),
    NewChannelReq(NewChannelReqPayload<'a>),
    NewChannelAns(NewChannelAnsPayload<'a>),
    RXTimingSetupReq(RXTimingSetupReqPayload<'a>),
    RXTimingSetupAns(RXTimingSetupAnsPayload),
    TXParamSetupReq(TXParamSetupReqPayload<'a>),
    TXParamSetupAns(TXParamSetupAnsPayload),
    DlChannelReq(DlChannelReqPayload<'a>),
    DlChannelAns(DlChannelAnsPayload),
    DeviceTimeReq(DeviceTimeReqPayload),
    DeviceTimeAns(DeviceTimeAnsPayload<'a>),
}

impl<'a> MacCommand<'a> {
    #![allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        match *self {
            MacCommand::LinkCheckReq(_) => LinkCheckReqPayload::len(),
            MacCommand::LinkCheckAns(_) => LinkCheckAnsPayload::len(),
            MacCommand::LinkADRReq(_) => LinkADRReqPayload::len(),
            MacCommand::LinkADRAns(_) => LinkADRAnsPayload::len(),
            MacCommand::DutyCycleReq(_) => DutyCycleReqPayload::len(),
            MacCommand::DutyCycleAns(_) => DutyCycleAnsPayload::len(),
            MacCommand::RXParamSetupReq(_) => RXParamSetupReqPayload::len(),
            MacCommand::RXParamSetupAns(_) => RXParamSetupAnsPayload::len(),
            MacCommand::DevStatusReq(_) => DevStatusReqPayload::len(),
            MacCommand::DevStatusAns(_) => DevStatusAnsPayload::len(),
            MacCommand::NewChannelReq(_) => NewChannelReqPayload::len(),
            MacCommand::NewChannelAns(_) => NewChannelAnsPayload::len(),
            MacCommand::RXTimingSetupReq(_) => RXTimingSetupReqPayload::len(),
            MacCommand::RXTimingSetupAns(_) => RXTimingSetupAnsPayload::len(),
            MacCommand::TXParamSetupReq(_) => TXParamSetupReqPayload::len(),
            MacCommand::TXParamSetupAns(_) => TXParamSetupAnsPayload::len(),
            MacCommand::DlChannelReq(_) => DlChannelReqPayload::len(),
            MacCommand::DlChannelAns(_) => DlChannelAnsPayload::len(),
            MacCommand::DeviceTimeReq(_) => DeviceTimeReqPayload::len(),
            MacCommand::DeviceTimeAns(_) => DeviceTimeAnsPayload::len(),
        }
    }
    pub fn uplink(&self) -> bool {
        match *self {
            MacCommand::LinkCheckReq(_) => LinkCheckReqPayload::uplink(),
            MacCommand::LinkCheckAns(_) => LinkCheckAnsPayload::uplink(),
            MacCommand::LinkADRReq(_) => LinkADRReqPayload::uplink(),
            MacCommand::LinkADRAns(_) => LinkADRAnsPayload::uplink(),
            MacCommand::DutyCycleReq(_) => DutyCycleReqPayload::uplink(),
            MacCommand::DutyCycleAns(_) => DutyCycleAnsPayload::uplink(),
            MacCommand::RXParamSetupReq(_) => RXParamSetupReqPayload::uplink(),
            MacCommand::RXParamSetupAns(_) => RXParamSetupAnsPayload::uplink(),
            MacCommand::DevStatusReq(_) => DevStatusReqPayload::uplink(),
            MacCommand::DevStatusAns(_) => DevStatusAnsPayload::uplink(),
            MacCommand::NewChannelReq(_) => NewChannelReqPayload::uplink(),
            MacCommand::NewChannelAns(_) => NewChannelAnsPayload::uplink(),
            MacCommand::RXTimingSetupReq(_) => RXTimingSetupReqPayload::uplink(),
            MacCommand::RXTimingSetupAns(_) => RXTimingSetupAnsPayload::uplink(),
            MacCommand::TXParamSetupReq(_) => TXParamSetupReqPayload::uplink(),
            MacCommand::TXParamSetupAns(_) => TXParamSetupAnsPayload::uplink(),
            MacCommand::DlChannelReq(_) => DlChannelReqPayload::uplink(),
            MacCommand::DlChannelAns(_) => DlChannelAnsPayload::uplink(),
            MacCommand::DeviceTimeReq(_) => DeviceTimeReqPayload::uplink(),
            MacCommand::DeviceTimeAns(_) => DeviceTimeAnsPayload::uplink(),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        match *self {
            MacCommand::LinkCheckReq(_) => &[],
            MacCommand::LinkCheckAns(ref v) => v.0,
            MacCommand::LinkADRReq(ref v) => v.0,
            MacCommand::LinkADRAns(ref v) => v.0,
            MacCommand::DutyCycleReq(ref v) => v.0,
            MacCommand::DutyCycleAns(_) => &[],
            MacCommand::RXParamSetupReq(ref v) => v.0,
            MacCommand::RXParamSetupAns(ref v) => v.0,
            MacCommand::DevStatusReq(_) => &[],
            MacCommand::DevStatusAns(ref v) => v.0,
            MacCommand::NewChannelReq(ref v) => v.0,
            MacCommand::NewChannelAns(ref v) => v.0,
            MacCommand::RXTimingSetupReq(ref v) => v.0,
            MacCommand::RXTimingSetupAns(_) => &[],
            MacCommand::TXParamSetupReq(ref v) => v.0,
            MacCommand::TXParamSetupAns(_) => &[],
            MacCommand::DlChannelReq(ref v) => v.0,
            MacCommand::DlChannelAns(_) => &[],
            MacCommand::DeviceTimeReq(_) => &[],
            MacCommand::DeviceTimeAns(ref v) => v.0,
        }
    }
}

impl<'a> SerializableMacCommand for MacCommand<'a> {
    fn payload_bytes(&self) -> &[u8] {
        self.bytes()
    }

    fn cid(&self) -> u8 {
        match *self {
            MacCommand::LinkCheckReq(_) => LinkCheckReqPayload::cid(),
            MacCommand::LinkCheckAns(_) => LinkCheckAnsPayload::cid(),
            MacCommand::LinkADRReq(_) => LinkADRReqPayload::cid(),
            MacCommand::LinkADRAns(_) => LinkADRAnsPayload::cid(),
            MacCommand::DutyCycleReq(_) => DutyCycleReqPayload::cid(),
            MacCommand::DutyCycleAns(_) => DutyCycleAnsPayload::cid(),
            MacCommand::RXParamSetupReq(_) => RXParamSetupReqPayload::cid(),
            MacCommand::RXParamSetupAns(_) => RXParamSetupAnsPayload::cid(),
            MacCommand::DevStatusReq(_) => DevStatusReqPayload::cid(),
            MacCommand::DevStatusAns(_) => DevStatusAnsPayload::cid(),
            MacCommand::NewChannelReq(_) => NewChannelReqPayload::cid(),
            MacCommand::NewChannelAns(_) => NewChannelAnsPayload::cid(),
            MacCommand::RXTimingSetupReq(_) => RXTimingSetupReqPayload::cid(),
            MacCommand::RXTimingSetupAns(_) => RXTimingSetupAnsPayload::cid(),
            MacCommand::TXParamSetupReq(_) => TXParamSetupReqPayload::cid(),
            MacCommand::TXParamSetupAns(_) => TXParamSetupAnsPayload::cid(),
            MacCommand::DlChannelReq(_) => DlChannelReqPayload::cid(),
            MacCommand::DlChannelAns(_) => DlChannelAnsPayload::cid(),
            MacCommand::DeviceTimeReq(_) => DeviceTimeReqPayload::cid(),
            MacCommand::DeviceTimeAns(_) => DeviceTimeAnsPayload::cid(),
        }
    }

    fn payload_len(&self) -> usize {
        self.len()
    }
}
