pub mod types;
use core::{fmt::Debug, future::Future};
use types::*;

/// An asynchronous timer that allows the state machine to await
/// between RX windows.

/// An asynchronous radio implementation that can transmit and receive data.
pub trait Radio: Sized {
    #[cfg(feature = "defmt")]
    type Error: Debug + defmt::Format;

    #[cfg(not(feature = "defmt"))]
    type Error: Debug;

    type TxFuture<'m>: Future<Output = Result<usize, Self::Error>> + 'm
    where
        Self: 'm;

    /// Transmit data buffer with the given tranciever configuration. The returned future
    /// should only complete once data have been transmitted.
    fn tx<'m>(&'m mut self, config: TxConfig, buf: &'m [u8]) -> Self::TxFuture<'m>;
    type RxFuture<'m>: Future<Output = Result<(usize, RxQuality), Self::Error>> + 'm
    where
        Self: 'm;
    /// Receive data into the provided buffer with the given tranciever configuration. The returned future
    /// should only complete when RX data have been received.
    fn rx<'m>(&'m mut self, config: RfConfig, rx_buf: &'m mut [u8]) -> Self::RxFuture<'m>;
}
