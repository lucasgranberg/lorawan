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

#[cfg(test)]
mod tests {
    use super::*;
    use heapless::Vec;
    #[test]
    fn test_build_command() {
        let mut cmds: Vec<UplinkMacCommandCreator, 15> = Vec::new();
        let mut ans = LinkADRAnsCreator::new();
        ans.set_channel_mask_ack(true);
        ans.set_data_rate_ack(true);
        ans.set_tx_power_ack(true);
        cmds.push(UplinkMacCommandCreator::LinkADRAns(ans)).unwrap();
        let mut buf: [u8; 4] = [0; 4];

        let mut dyn_cmds: Vec<&dyn SerializableMacCommand, 8> = Vec::new();
        for cmd in cmds.iter() {
            if let Err(_e) = dyn_cmds.push(cmd) {
                panic!("dyn_cmds too small compared to cmds")
            }
        }

        assert_eq!(dyn_cmds.first().unwrap().payload_len(), 1);

        build_mac_commands(&dyn_cmds, &mut buf).unwrap();

        assert_eq!(buf, [0x03, 0x07, 0x00, 0x00])
    }
}
