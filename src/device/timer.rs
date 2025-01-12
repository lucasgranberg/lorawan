//! Timer functionality which must be implemented by calling code.

use core::fmt::Debug;

/// An asynchronous timer that allows the state machine to await
/// between RX windows.
pub trait Timer: Sized {
    /// Possible result error.
    #[cfg(feature = "defmt")]
    type Error: Debug + defmt::Format;
    #[cfg(not(feature = "defmt"))]
    type Error: Debug;

    /// Reset the timer.
    fn reset(&mut self);
    /// Set the timer to notify in the future.
    async fn at(&self, millis: u64) -> Result<(), Self::Error>;
}
