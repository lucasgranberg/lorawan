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
