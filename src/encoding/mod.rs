// Copyright (c) 2017,2018,2020 Ivaylo Petrov
//
// Licensed under the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//
// author: Ivaylo Petrov <ivajloip@gmail.com>

//! LoRaWAN packet handling and parsing.

#![allow(clippy::upper_case_acronyms)]

use crate::device::Device;
pub mod creator;
pub mod keys;
pub mod maccommandcreator;
pub mod maccommands;
pub mod parser;

pub mod default_crypto;

mod securityhelpers;

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(missing_docs)]
pub enum Error {
    OutOfRange,
    BufferTooSmall,
    IncorrectSizeForMacCommand,
    InvalidDataRange,
    InvalidKey,
    InvalidData,
    InvalidMic,
    InvalidChannelIndex,
    UnsupportedMessageType,
    PhyDataEmpty,
    InsufficeientNumberOfBytes,
    UnsupportedMajorVersion,
    MacCommandTooBigForFOpts,
    DataAndMacCommandsInPayloadNotAllowed,
    FRMPayloadWithFportZero,
    CfListTooLong,
}
impl<D> From<Error> for super::Error<D>
where
    D: Device,
{
    fn from(value: Error) -> Self {
        Self::Encoding(value)
    }
}
