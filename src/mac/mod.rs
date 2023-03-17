use core::fmt::Debug;
use futures::Future;

use crate::{
    device::{radio_buffer::RadioBuffer, Device},
    Window,
};

use self::mac_1_0_4::region::Region;

pub mod mac_1_0_4;
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    UnsupportedDataRate,
    InvalidMic,
    InvalidDevAddr,
    InvalidPayloadType,
    NetworkNotJoined,
    SessionExpired,
    FOptsFull,
    NoValidChannelFound,
}
pub trait Mac<R, D>
where
    R: Region,
    D: Device,
{
    type Error;

    type JoinFuture<'m>: Future<Output = Result<(), Self::Error>> + 'm
    where
        Self: 'm,
        D: 'm;
    type SendFuture<'m>: Future<Output = Result<usize, Self::Error>> + 'm
    where
        Self: 'm,
        D: 'm;

    fn join<'m>(
        &'m mut self,
        device: &'m mut D,
        radio_buffer: &'m mut RadioBuffer<256>,
    ) -> Self::JoinFuture<'m>;

    fn send<'m>(
        &'m mut self,
        device: &'m mut D,
        radio_buffer: &'m mut RadioBuffer<256>,
        data: &'m [u8],
        fport: u8,
        confirmed: bool,
        rx: Option<&'m mut [u8]>,
    ) -> Self::SendFuture<'m>;
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
