#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(type_alias_impl_trait)]
#![cfg_attr(test, feature(impl_trait_in_assoc_type))]
#![feature(concat_idents)]
#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use core::fmt::Debug;
use device::Device;
use mac::region;

pub mod device;
pub mod encoding;
pub mod mac;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(missing_docs)]
pub enum Error<D>
where
    D: Device,
{
    Device(device::Error<D>),
    Region(region::Error),
    Mac(mac::Error),
    Encoding(encoding::Error),
}

#[cfg(test)]
pub(crate) mod tests {
    use core::convert::Infallible;
    use core::future::Future;

    use super::*;
    use crate::device::non_volatile_store::NonVolatileStore;
    use crate::device::packet_buffer::PacketBuffer;
    use crate::device::packet_queue::{PacketQueue, PACKET_SIZE};
    use crate::device::radio::types::{RfConfig, RxQuality, TxConfig};
    use crate::device::radio::Radio;
    use crate::device::rng::Rng;
    use crate::device::timer::Timer;
    use crate::mac::types::Storable;

    #[derive(Debug, PartialEq, defmt::Format)]
    pub(crate) struct RadioMock {}
    impl Radio for RadioMock {
        type Error = Infallible;
        async fn tx(&mut self, _config: TxConfig, _buf: &[u8]) -> Result<usize, Self::Error> {
            Ok(0)
        }
        async fn rx(
            &mut self,
            _config: RfConfig,
            _window_in_secs: u8,
            _rx_buf: &mut [u8],
        ) -> Result<(usize, RxQuality), Self::Error> {
            Ok((0, RxQuality { rssi: 0, snr: 0 }))
        }

        async fn sleep(&mut self, _warm_start: bool) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[derive(Debug, PartialEq, defmt::Format)]
    pub(crate) enum NonVolatileStoreError {
        Encoding,
    }
    #[derive(Debug, PartialEq, defmt::Format)]
    pub(crate) struct NonVolatileStoreMock {}
    impl NonVolatileStore for NonVolatileStoreMock {
        type Error = NonVolatileStoreError;

        fn save(&mut self, _storable: Storable) -> Result<(), Self::Error> {
            Err(NonVolatileStoreError::Encoding)
        }

        fn load(&mut self) -> Result<Storable, Self::Error> {
            Err(NonVolatileStoreError::Encoding)
        }
    }

    #[derive(Debug, PartialEq, defmt::Format)]
    pub(crate) struct RngMock {}
    impl Rng for RngMock {
        type Error = Infallible;

        fn next_u32(&mut self) -> Result<u32, Self::Error> {
            Ok(42u32)
        }
    }

    #[derive(Debug, PartialEq, defmt::Format)]
    pub(crate) struct TimerMock {}
    impl Timer for TimerMock {
        type Error = Infallible;
        type AtFuture<'a> = impl Future<Output = ()> + 'a where Self: 'a;

        fn reset(&mut self) {}
        fn at<'a>(&self, _millis: u64) -> Result<Self::AtFuture<'a>, Self::Error> {
            let fut = async move {};
            Ok(fut) as Result<Self::AtFuture<'a>, Infallible>
        }
    }

    #[derive(Debug, PartialEq, defmt::Format)]
    pub(crate) struct PacketQueueMock {}
    impl PacketQueue for PacketQueueMock {
        type Error = Infallible;

        async fn push(&mut self, _packet: PacketBuffer<PACKET_SIZE>) -> Result<(), Self::Error> {
            Ok(())
        }
        async fn next(&mut self) -> Result<PacketBuffer<PACKET_SIZE>, Self::Error> {
            Ok(PacketBuffer::default())
        }
        fn available(&mut self) -> Result<bool, Self::Error> {
            Ok(false)
        }
    }

    #[derive(Debug, PartialEq, defmt::Format)]
    pub(crate) struct DeviceMock {
        rng: RngMock,
        radio: RadioMock,
        timer: TimerMock,
        non_volatile_store: NonVolatileStoreMock,
        uplink_packet_queue: PacketQueueMock,
        downlink_packet_queue: PacketQueueMock,
    }

    impl DeviceMock {
        #[allow(dead_code)]
        pub(crate) fn new() -> Self {
            Self {
                rng: RngMock {},
                radio: RadioMock {},
                timer: TimerMock {},
                non_volatile_store: NonVolatileStoreMock {},
                uplink_packet_queue: PacketQueueMock {},
                downlink_packet_queue: PacketQueueMock {},
            }
        }
    }

    impl Device for DeviceMock {
        type Timer = TimerMock;

        type Radio = RadioMock;

        type Rng = RngMock;

        type NonVolatileStore = NonVolatileStoreMock;

        type PacketQueue = PacketQueueMock;

        fn timer(&mut self) -> &mut Self::Timer {
            &mut self.timer
        }

        fn radio(&mut self) -> &mut Self::Radio {
            &mut self.radio
        }

        fn rng(&mut self) -> &mut Self::Rng {
            &mut self.rng
        }

        fn non_volatile_store(&mut self) -> &mut Self::NonVolatileStore {
            &mut self.non_volatile_store
        }

        fn uplink_packet_queue(&mut self) -> &mut Self::PacketQueue {
            &mut self.uplink_packet_queue
        }

        fn downlink_packet_queue(&mut self) -> &mut Self::PacketQueue {
            &mut self.downlink_packet_queue
        }

        fn max_eirp() -> u8 {
            22
        }

        fn adaptive_data_rate_enabled(&self) -> bool {
            true
        }

        fn handle_device_time(&mut self, _seconds: u32, _nano_seconds: u32) {}

        fn handle_link_check(&mut self, _gateway_count: u8, _margin: u8) {}

        fn battery_level(&self) -> Option<f32> {
            None
        }
    }
}
