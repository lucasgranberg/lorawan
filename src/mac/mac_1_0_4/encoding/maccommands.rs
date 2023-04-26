use crate::encoding::maccommands::*;

mac_cmds_enum! {
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

mac_cmds_enum! {
    pub enum UplinkMacCommand<'a> {
        LinkCheckReq(LinkCheckReqPayload)
        LinkADRAns(LinkADRAnsPayload<'a>)
        DutyCycleAns(DutyCycleAnsPayload)
        RXParamSetupAns(RXParamSetupAnsPayload<'a>)
        DevStatusAns(DevStatusAnsPayload<'a>)
        NewChannelAns(NewChannelAnsPayload<'a>)
        RXTimingSetupAns(RXTimingSetupAnsPayload)
        TXParamSetupAns(TXParamSetupAnsPayload)
        DlChannelAns(DlChannelAnsPayload<'a>)
        DeviceTimeReq(DeviceTimeReqPayload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_command() {
        let data = [77, 251, 3, 224, 161, 0, 0, DevStatusReqPayload::cid()];
        let fhdr: crate::encoding::parser::FHDR<'_> = crate::encoding::parser::FHDR(&data, true);
        let mut iterator: MacCommandIterator<'_, DownlinkMacCommand> = (&fhdr).into();
        assert_eq!(
            iterator.next(),
            Some(DownlinkMacCommand::DevStatusReq(DevStatusReqPayload()))
        );
        assert_eq!(iterator.next(), None)
    }
}
