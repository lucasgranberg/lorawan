#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(type_alias_impl_trait)]
#![feature(concat_idents)]
#![feature(async_fn_in_trait)]

use core::fmt::Debug;
mod fmt;

use device::Device;
use mac::region;

pub mod device;
pub mod encoding;
pub mod mac;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<D>
where
    D: Device,
{
    Device(device::Error<D>),
    Region(region::Error),
    Mac(mac::Error),
    Encoding(encoding::Error),
}
