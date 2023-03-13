use futures::Future;

use crate::{
    device::{
        radio::types::{CodingRate, Datarate, RadioBuffer},
        Device,
    },
    Window, DR,
};

pub mod mac_1_0_4;

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

pub trait Region {
    fn default_channels() -> u8;
    fn min_data_rate() -> DR;
    fn max_data_rate() -> DR;
    fn default_data_rate() -> DR;
    fn default_coding_rate() -> CodingRate;
    fn default_rx2_frequency() -> u32;
    fn default_rx2_data_rate() -> DR;
    fn max_eirp() -> i8;
    fn min_frequency() -> u32;
    fn max_frequency() -> u32;
    fn convert_data_rate(dr: DR) -> Datarate;
    fn get_receive_window(rx_dr_offset: DR, downstream_dr: DR) -> DR;
    fn supports_tx_param_setup() -> bool;
    fn modify_dbm(tx_power: u8, cur_dbm: Option<i8>, max_eirp: i8) -> Result<Option<i8>, ()>;

    fn default_receive_delay1() -> u32 {
        1000
    }
    fn default_receive_delay2() -> u32 {
        Self::default_receive_delay1() + 1
    }
    fn default_rx1_data_rate_offset() -> DR {
        DR::_0
    }
    fn default_join_accept_delay1() -> u32 {
        5000
    }
    fn default_join_accept_delay2() -> u32 {
        Self::default_join_accept_delay1() + 1
    }
    fn default_max_fcnt_gap() -> u32 {
        16384
    }
    fn default_adr_ack_limit() -> usize {
        64
    }
    fn default_adr_ack_delay() -> u8 {
        32
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
