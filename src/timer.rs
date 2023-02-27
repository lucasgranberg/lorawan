use core::future::Future;

pub trait Timer {
    fn reset(&mut self);

    type AtFuture<'m>: Future<Output = ()> + 'm
    where
        Self: 'm;

    fn at(&mut self, millis: u64) -> Self::AtFuture<'_>;

    type DelayFuture: Future<Output = ()>;
    /// Delay for millis milliseconds
    fn delay_ms(&mut self, millis: u64) -> Self::DelayFuture;
}
