use crate::encoding::{
    maccommandcreator::{DlChannelAnsCreator, NewChannelAnsCreator},
    maccommands::{DlChannelReqPayload, NewChannelReqPayload},
};

use self::channel_plan::ChannelPlan;
pub mod channel_plan;

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
