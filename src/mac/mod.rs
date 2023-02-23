use futures::Future;

use crate::{radio::types::RadioBuffer, timer::Timer, MType, Window, DR};

mod mac_1_0_4;

pub trait Mac<R, P, T>
where
    R: Region,
    P: crate::radio::PhyRxTx,
    T: Timer,
{
    type Error;

    type JoinFuture<'m>: Future<Output = Result<(), Self::Error>> + 'm
    where
        Self: 'm,
        P: 'm;
    type SendFuture<'m>: Future<Output = Result<usize, Self::Error>> + 'm
    where
        Self: 'm,
        P: 'm;

    fn join<'m>(
        &'m mut self,
        radio: &'m mut P,
        radio_buffer: &'m mut RadioBuffer<256>,
    ) -> Self::JoinFuture<'m>;

    fn send<'m>(
        &'m mut self,
        radio: &'m mut P,
        radio_buffer: &'m mut RadioBuffer<256>,
        data: &'m [u8],
        fport: u8,
        confirmed: bool,
        rx: Option<&'m mut [u8]>,
    ) -> Self::SendFuture<'m>;
}

pub trait Region {
    fn get_default_datarate() -> DR;
    fn create_tx_config(frame_type: MType, data_rate: DR) -> crate::radio::types::TxConfig;
    fn create_rf_config(frame_type: MType, data_rate: DR) -> crate::radio::types::RfConfig;
}

struct Timings {
    rx1_open: u16,
    rx1_close: u16,
    rx2_open: u16,
    rx2_close: u16,
}
impl Timings {
    pub fn get_open(&self, window: &Window) -> u16 {
        match window {
            Window::_1 => self.rx1_open,
            Window::_2 => self.rx2_open,
        }
    }
    pub fn get_close(&self, window: &Window) -> u16 {
        match window {
            Window::_1 => self.rx1_close,
            Window::_2 => self.rx2_close,
        }
    }
}
