use core::future::Future;

pub trait Timer {
    fn reset(&mut self);

    type AtFuture: Future<Output = ()>;

    fn at(&mut self, millis: u64) -> Self::AtFuture;

    type DelayFuture: Future<Output = ()>;
    /// Delay for millis milliseconds
    fn delay_ms(&mut self, millis: u64) -> Self::DelayFuture;
}
