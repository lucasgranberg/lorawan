pub mod channel_plan;

struct RegionalChannelMask {
    index: u8,
    enabled: bool,
}

pub trait Region: crate::mac::Region {}

pub mod eu868;
