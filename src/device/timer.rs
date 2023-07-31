//! Timer functionality which must be implemented by calling code.

use core::{fmt::Debug, future::Future};

/// An asynchronous timer that allows the state machine to await
/// between RX windows.
pub trait Timer: Sized {
    /// Possible result error.
    #[cfg(feature = "defmt")]
    type Error: Debug + defmt::Format;

    #[cfg(not(feature = "defmt"))]
    type Error: Debug;
    fn reset(&mut self);

    /// Notification of reaching a time point.
    type AtFuture<'m>: Future<Output = ()> + 'm
    where
        Self: 'm;

    fn at<'a>(&self, millis: u64) -> Result<Self::AtFuture<'a>, Self::Error>;
}
