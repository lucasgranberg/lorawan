use futures::Future;

use crate::{
    device::{
        self,
        radio::types::{CodingRate, RadioBuffer},
        Device,
    },
    Frame, Window, DR,
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
    fn default_datarate() -> DR {
        DR::_0
    }
    fn default_coding_rate() -> CodingRate {
        CodingRate::_4_5
    }
    fn max_eirp() -> i8 {
        14
    }
    fn supports_tx_param_setup() -> bool {
        false
    }
    /// Convert tx power to dbm
    fn modify_dbm(tx_power: u8, cur_dbm: Option<i8>, max_eirp: i8) -> Result<Option<i8>, ()>;

    fn create_rf_config(
        frame: Frame,
        random: u32,
        data_rate: Option<DR>,
    ) -> device::radio::types::RfConfig;
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