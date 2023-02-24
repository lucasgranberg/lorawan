use crate::encoding::{maccommandcreator::*, maccommands::*};

#[derive(Debug, PartialEq, Eq)]
pub enum MacCommandCreator {
    LinkCheckReq(LinkCheckReqCreator),
    LinkCheckAns(LinkCheckAnsCreator),
    LinkADRReq(LinkADRReqCreator),
    LinkADRAns(LinkADRAnsCreator),
    DutyCycleReq(DutyCycleReqCreator),
    DutyCycleAns(DutyCycleAnsCreator),
    RXParamSetupReq(RXParamSetupReqCreator),
    RXParamSetupAns(RXParamSetupAnsCreator),
    DevStatusReq(DevStatusReqCreator),
    DevStatusAns(DevStatusAnsCreator),
    NewChannelReq(NewChannelReqCreator),
    NewChannelAns(NewChannelAnsCreator),
    RXTimingSetupReq(RXTimingSetupReqCreator),
    RXTimingSetupAns(RXTimingSetupAnsCreator),
    TXParamSetupReq(TXParamSetupReqCreator),
    TXParamSetupAns(TXParamSetupAnsCreator),
    DlChannelReq(DlChannelReqCreator),
    DlChannelAns(DlChannelAnsCreator),
    DeviceTimeReq(DeviceTimeReqCreator),
    DeviceTimeAns(DeviceTimeAnsCreator),
}

impl MacCommandCreator {
    #![allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        match *self {
            Self::LinkCheckReq(_) => LinkCheckReqPayload::len(),
            Self::LinkCheckAns(_) => LinkCheckAnsPayload::len(),
            Self::LinkADRReq(_) => LinkADRReqPayload::len(),
            Self::LinkADRAns(_) => LinkADRAnsPayload::len(),
            Self::DutyCycleReq(_) => DutyCycleReqPayload::len(),
            Self::DutyCycleAns(_) => DutyCycleAnsPayload::len(),
            Self::RXParamSetupReq(_) => RXParamSetupReqPayload::len(),
            Self::RXParamSetupAns(_) => RXParamSetupAnsPayload::len(),
            Self::DevStatusReq(_) => DevStatusReqPayload::len(),
            Self::DevStatusAns(_) => DevStatusAnsPayload::len(),
            Self::NewChannelReq(_) => NewChannelReqPayload::len(),
            Self::NewChannelAns(_) => NewChannelAnsPayload::len(),
            Self::RXTimingSetupReq(_) => RXTimingSetupReqPayload::len(),
            Self::RXTimingSetupAns(_) => RXTimingSetupAnsPayload::len(),
            Self::TXParamSetupReq(_) => TXParamSetupReqPayload::len(),
            Self::TXParamSetupAns(_) => TXParamSetupAnsPayload::len(),
            Self::DlChannelReq(_) => DlChannelReqPayload::len(),
            Self::DlChannelAns(_) => DlChannelAnsPayload::len(),
            Self::DeviceTimeReq(_) => DeviceTimeReqPayload::len(),
            Self::DeviceTimeAns(_) => DeviceTimeAnsPayload::len(),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        match *self {
            Self::LinkCheckReq(_) => &[],
            Self::LinkCheckAns(ref v) => v.build(),
            Self::LinkADRReq(ref v) => v.build(),
            Self::LinkADRAns(ref v) => v.build(),
            Self::DutyCycleReq(ref v) => v.build(),
            Self::DutyCycleAns(_) => &[],
            Self::RXParamSetupReq(ref v) => v.build(),
            Self::RXParamSetupAns(ref v) => v.build(),
            Self::DevStatusReq(_) => &[],
            Self::DevStatusAns(ref v) => v.build(),
            Self::NewChannelReq(ref v) => v.build(),
            Self::NewChannelAns(ref v) => v.build(),
            Self::RXTimingSetupReq(ref v) => v.build(),
            Self::RXTimingSetupAns(_) => &[],
            Self::TXParamSetupReq(ref v) => v.build(),
            Self::TXParamSetupAns(_) => &[],
            Self::DlChannelReq(ref v) => v.build(),
            Self::DlChannelAns(_) => &[],
            Self::DeviceTimeReq(_) => &[],
            Self::DeviceTimeAns(ref v) => v.build(),
        }
    }
}
impl SerializableMacCommand for MacCommandCreator {
    fn payload_bytes(&self) -> &[u8] {
        self.bytes()
    }

    fn cid(&self) -> u8 {
        match *self {
            MacCommandCreator::LinkCheckReq(_) => LinkCheckReqPayload::cid(),
            MacCommandCreator::LinkCheckAns(_) => LinkCheckAnsPayload::cid(),
            MacCommandCreator::LinkADRReq(_) => LinkADRReqPayload::cid(),
            MacCommandCreator::LinkADRAns(_) => LinkADRAnsPayload::cid(),
            MacCommandCreator::DutyCycleReq(_) => DutyCycleReqPayload::cid(),
            MacCommandCreator::DutyCycleAns(_) => DutyCycleAnsPayload::cid(),
            MacCommandCreator::RXParamSetupReq(_) => RXParamSetupReqPayload::cid(),
            MacCommandCreator::RXParamSetupAns(_) => RXParamSetupAnsPayload::cid(),
            MacCommandCreator::DevStatusReq(_) => DevStatusReqPayload::cid(),
            MacCommandCreator::DevStatusAns(_) => DevStatusAnsPayload::cid(),
            MacCommandCreator::NewChannelReq(_) => NewChannelReqPayload::cid(),
            MacCommandCreator::NewChannelAns(_) => NewChannelAnsPayload::cid(),
            MacCommandCreator::RXTimingSetupReq(_) => RXTimingSetupReqPayload::cid(),
            MacCommandCreator::RXTimingSetupAns(_) => RXTimingSetupAnsPayload::cid(),
            MacCommandCreator::TXParamSetupReq(_) => TXParamSetupReqPayload::cid(),
            MacCommandCreator::TXParamSetupAns(_) => TXParamSetupAnsPayload::cid(),
            MacCommandCreator::DlChannelReq(_) => DlChannelReqPayload::cid(),
            MacCommandCreator::DlChannelAns(_) => DlChannelAnsPayload::cid(),
            MacCommandCreator::DeviceTimeReq(_) => DeviceTimeReqPayload::cid(),
            MacCommandCreator::DeviceTimeAns(_) => DeviceTimeAnsPayload::cid(),
        }
    }

    fn payload_len(&self) -> usize {
        self.len()
    }
}
