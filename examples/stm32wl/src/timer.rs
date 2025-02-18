use core::convert::Infallible;

use embassy_time::{Duration, Instant, Timer};

pub struct LoraTimer {
    start: Instant,
}
impl LoraTimer {
    pub fn new() -> Self {
        Self { start: Instant::now() }
    }
}
impl Default for LoraTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl ::lorawan::device::timer::Timer for LoraTimer {
    type Error = Infallible;

    fn reset(&mut self) {
        self.start = Instant::now();
    }

    async fn at(&self, millis: u64) -> Result<(), Self::Error> {
        let fut = Timer::at(self.start + Duration::from_millis(millis)).await;
        Ok(fut)
    }
}
