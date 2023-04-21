use core::fmt::Debug;

use crate::{device::Device, Window};

use self::mac_1_0_4::region::Region;

pub mod mac_1_0_4;
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    UnsupportedDataRate,
    InvalidMic,
    InvalidDevAddr,
    InvalidPayloadType,
    NoResponse,
    NetworkNotJoined,
    SessionExpired,
    FOptsFull,
    NoValidChannelFound,
}
impl<D> From<Error> for super::Error<D>
where
    D: Device,
{
    fn from(value: Error) -> Self {
        Self::Mac(value)
    }
}

pub(crate) struct RxWindows {
    rx1_open: u16,
    rx1_close: u16,
    rx2_open: u16,
    rx2_close: u16,
}
impl RxWindows {
    pub(crate) fn get_open(&self, window: &Window) -> u16 {
        match window {
            Window::_1 => self.rx1_open,
            Window::_2 => self.rx2_open,
        }
    }
    pub(crate) fn get_close(&self, window: &Window) -> u16 {
        match window {
            Window::_1 => self.rx1_close,
            Window::_2 => self.rx2_close,
        }
    }
}
