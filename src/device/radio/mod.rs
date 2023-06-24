pub mod types;
use core::fmt::Debug;
use types::*;

/// An asynchronous timer that allows the state machine to await
/// between RX windows.

/// An asynchronous radio implementation that can transmit and receive data.
pub trait Radio: Sized {
    #[cfg(feature = "defmt")]
    type Error: Debug + defmt::Format;

    #[cfg(not(feature = "defmt"))]
    type Error: Debug;

    /// Transmit data buffer with the given tranciever configuration. The returned future
    /// should only complete once data have been transmitted.
    async fn tx(&mut self, config: TxConfig, buf: &[u8]) -> Result<usize, Self::Error>;

    /// Receive data into the provided buffer with the given tranciever configuration. The returned future
    /// should only complete when RX data have been received or a timeout has occurred.
    async fn rx(
        &mut self,
        config: RfConfig,
        window_in_secs: u8,
        rx_buf: &mut [u8],
    ) -> Result<(usize, RxQuality), Self::Error>;

    /// Place the radio in sleep mode with warm or cold start specified.
    async fn sleep(&mut self, warm_start: bool) -> Result<(), Self::Error>;
}
