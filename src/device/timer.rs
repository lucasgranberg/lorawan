use core::{fmt::Debug, future::Future};

pub trait Timer {
    #[cfg(feature = "defmt")]
    type Error: Debug + defmt::Format;

    #[cfg(not(feature = "defmt"))]
    type Error: Debug;
    fn reset(&mut self);

    type AtFuture: Future<Output = Result<(), Self::Error>>;

    fn at(&mut self, millis: u64) -> Self::AtFuture;

    type DelayFuture: Future<Output = Result<(), Self::Error>>;
    /// Delay for millis milliseconds
    fn delay_ms(&mut self, millis: u64) -> Self::DelayFuture;
}
