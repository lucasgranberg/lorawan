use crate::{
    channel_mask::ChannelMask,
    encoding::{
        maccommandcreator::{DlChannelAnsCreator, NewChannelAnsCreator},
        maccommands::{DlChannelReqPayload, NewChannelReqPayload},
    },
};

use self::channel_plan::ChannelPlan;
pub mod channel_plan;

struct RegionalChannelMask {
    index: u8,
    enabled: bool,
}

pub trait Region<C>: crate::mac::Region
where
    C: ChannelPlan,
{
    fn handle_new_channel_req(
        channel_plan: &mut C,
        payload: NewChannelReqPayload,
    ) -> Option<NewChannelAnsCreator>;
    fn handle_dl_channel_req(
        channel_plan: &mut C,
        payload: DlChannelReqPayload,
    ) -> Option<DlChannelAnsCreator>;
}

pub mod eu868;
