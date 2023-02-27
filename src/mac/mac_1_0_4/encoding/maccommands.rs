use crate::encoding::maccommands::*;

mac_commands! {
    pub enum DownlinkMacCommand<'a> {
        LinkCheckAns(LinkCheckAnsPayload<'a>)
        LinkADRReq(LinkADRReqPayload<'a>)
        DutyCycleReq(DutyCycleReqPayload<'a>)
        RXParamSetupReq(RXParamSetupReqPayload<'a>)
        DevStatusReq(DevStatusReqPayload)
        NewChannelReq(NewChannelReqPayload<'a>)
        RXTimingSetupReq(RXTimingSetupReqPayload<'a>)
        TXParamSetupReq(TXParamSetupReqPayload<'a>)
        DlChannelReq(DlChannelReqPayload<'a>)
        DeviceTimeAns(DeviceTimeAnsPayload<'a>)
    }
}

mac_commands! {
    pub enum UplinkMacCommand<'a> {
        LinkCheckReq(LinkCheckReqPayload)
        LinkADRAns(LinkADRAnsPayload<'a>)
        DutyCycleAns(DutyCycleAnsPayload)
        RXParamSetupAns(RXParamSetupAnsPayload<'a>)
        DevStatusAns(DevStatusAnsPayload<'a>)
        NewChannelAns(NewChannelAnsPayload<'a>)
        RXTimingSetupAns(RXTimingSetupAnsPayload)
        TXParamSetupAns(TXParamSetupAnsPayload)
        DlChannelAns(DlChannelAnsPayload)
        DeviceTimeReq(DeviceTimeReqPayload)
    }
}
