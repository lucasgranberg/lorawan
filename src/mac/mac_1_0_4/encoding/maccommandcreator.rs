use crate::encoding::{maccommandcreator::*, maccommands::*};

mac_cmds_creator_enum! {
    pub enum UplinkMacCommandCreator {
        LinkCheckReq
        LinkADRAns
        DutyCycleAns
        RXParamSetupAns
        DevStatusAns
        NewChannelAns
        RXTimingSetupAns
        TXParamSetupAns
        DlChannelAns
        DeviceTimeReq
    }
}
mac_cmds_creator_enum! {
    pub enum DownlinkMacCommandCreator {
        LinkCheckAns
        LinkADRReq
        DutyCycleReq
        RXParamSetupReq
        DevStatusReq
        NewChannelReq
        RXTimingSetupReq
        TXParamSetupReq
        DlChannelReq
        DeviceTimeAns
    }
}
