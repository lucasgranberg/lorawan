use core::{fmt::Debug, future::Future};

pub trait Timer: Sized {
    #[cfg(feature = "defmt")]
    type Error: Debug + defmt::Format;

    #[cfg(not(feature = "defmt"))]
    type Error: Debug;
    fn reset(&mut self);

    type AtFuture: Future<Output = ()>;

    fn at(&mut self, millis: u64) -> Result<Self::AtFuture, Self::Error>;

    // type DelayFuture: Future<Output = Result<(), Self::Error>>;
    // /// Delay for millis milliseconds
    // fn delay_ms(&mut self, millis: u64) -> Self::DelayFuture;
}
