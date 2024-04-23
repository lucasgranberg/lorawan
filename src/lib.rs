#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, feature(impl_trait_in_assoc_type))]
#![allow(async_fn_in_trait)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use core::fmt::Debug;
use device::Device;
use mac::region;

pub mod device;
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
    Encoding,
}
