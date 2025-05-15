#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![allow(async_fn_in_trait)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use core::fmt::Debug;
use device::Device;
use mac::region;

// fmt comes first
pub(crate) mod fmt;

pub mod device;
pub mod mac;
pub use encoding;
pub use lora_modulation as modulation;
pub use lora_phy as phy;

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
}
