use crate::encoding::{maccommandcreator::*, maccommands::*};

#[derive(Debug, PartialEq, Eq)]
pub enum UplinkMacCommandCreator {
    LinkCheckReq(LinkCheckReqCreator),
    LinkADRAns(LinkADRAnsCreator),
    DutyCycleAns(DutyCycleAnsCreator),
    RXParamSetupAns(RXParamSetupAnsCreator),
    DevStatusAns(DevStatusAnsCreator),
    NewChannelAns(NewChannelAnsCreator),
    RXTimingSetupAns(RXTimingSetupAnsCreator),
    TXParamSetupAns(TXParamSetupAnsCreator),
    DlChannelAns(DlChannelAnsCreator),
    DeviceTimeReq(DeviceTimeReqCreator),
}
#[derive(Debug, PartialEq, Eq)]
pub enum DownlinkMacCommandCreator {
    LinkCheckAns(LinkCheckAnsCreator),
    LinkADRReq(LinkADRReqCreator),
    DutyCycleReq(DutyCycleReqCreator),
    RXParamSetupReq(RXParamSetupReqCreator),
    DevStatusReq(DevStatusReqCreator),
    NewChannelReq(NewChannelReqCreator),
    RXTimingSetupReq(RXTimingSetupReqCreator),
    TXParamSetupReq(TXParamSetupReqCreator),
    DlChannelReq(DlChannelReqCreator),
    DeviceTimeAns(DeviceTimeAnsCreator),
}
impl UplinkMacCommandCreator {
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

    pub fn bytes(&self) -> &[u8] {
        match *self {
            Self::LinkCheckReq(_) => &[],
            Self::LinkADRAns(ref v) => v.build(),
            Self::DutyCycleAns(_) => &[],
            Self::RXParamSetupAns(ref v) => v.build(),
            Self::DevStatusAns(ref v) => v.build(),
            Self::NewChannelAns(ref v) => v.build(),
            Self::RXTimingSetupAns(_) => &[],
            Self::TXParamSetupAns(_) => &[],
            Self::DlChannelAns(_) => &[],
            Self::DeviceTimeReq(_) => &[],
        }
    }
}
impl DownlinkMacCommandCreator {
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

    pub fn bytes(&self) -> &[u8] {
        match *self {
            Self::LinkCheckAns(ref v) => v.build(),
            Self::LinkADRReq(ref v) => v.build(),
            Self::DutyCycleReq(ref v) => v.build(),
            Self::RXParamSetupReq(ref v) => v.build(),
            Self::DevStatusReq(_) => &[],
            Self::NewChannelReq(ref v) => v.build(),
            Self::RXTimingSetupReq(ref v) => v.build(),
            Self::TXParamSetupReq(ref v) => v.build(),
            Self::DlChannelReq(ref v) => v.build(),
            Self::DeviceTimeAns(ref v) => v.build(),
        }
    }
}
impl SerializableMacCommand for UplinkMacCommandCreator {
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
impl SerializableMacCommand for DownlinkMacCommandCreator {
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
