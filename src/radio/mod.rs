pub mod types;
use core::future::Future;
use types::*;
pub struct RadioError;

/// An asynchronous timer that allows the state machine to await
/// between RX windows.

/// An asynchronous radio implementation that can transmit and receive data.
pub trait PhyRxTx: Sized {
    #[cfg(feature = "defmt")]
    type PhyError: defmt::Format;

    #[cfg(not(feature = "defmt"))]
    type PhyError;

    type TxFuture<'m>: Future<Output = Result<usize, Self::PhyError>> + 'm
    where
        Self: 'm;

    /// Transmit data buffer with the given tranciever configuration. The returned future
    /// should only complete once data have been transmitted.
    fn tx<'m>(&self, config: TxConfig, buf: &'m [u8]) -> Self::TxFuture<'m>;

    type RxFuture<'m>: Future<Output = Result<(usize, RxQuality), Self::PhyError>> + 'm
    where
        Self: 'm;
    /// Receive data into the provided buffer with the given tranciever configuration. The returned future
    /// should only complete when RX data have been received.
    fn rx<'m>(&self, config: RfConfig, rx_buf: &'m mut [u8]) -> Self::RxFuture<'m>;
}
