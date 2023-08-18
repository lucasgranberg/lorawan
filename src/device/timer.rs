//! Timer functionality which must be implemented by calling code.

use core::{fmt::Debug, future::Future};

/// An asynchronous timer that allows the state machine to await
/// between RX windows.
pub trait Timer: Sized {
    /// Notification of reaching a time point.
    type AtFuture<'a>: Future<Output = ()> + 'a
    where
        Self: 'a;
    /// Possible result error.
    #[cfg(feature = "defmt")]
    type Error: Debug + defmt::Format;
    #[cfg(not(feature = "defmt"))]
    type Error: Debug;

    /// Reset the timer.
    fn reset(&mut self);
    /// Set the timer to notify in the future.
    fn at<'a>(&self, millis: u64) -> Result<Self::AtFuture<'a>, Self::Error>;
}
