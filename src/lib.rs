#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(type_alias_impl_trait)]
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
