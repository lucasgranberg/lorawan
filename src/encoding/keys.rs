// Copyright (c) 2017-2020 Ivaylo Petrov
//
// Licensed under the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//
// author: Ivaylo Petrov <ivajloip@gmail.com>

//! Specification of the encryption/decryption features for keys used by LoRaWAN packets.

use super::securityhelpers::generic_array::{typenum::U16, GenericArray};

/// AES128 represents 128 bit AES key.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct AES128(pub [u8; 16]);

impl From<[u8; 16]> for AES128 {
    fn from(v: [u8; 16]) -> Self {
        AES128(v)
    }
}

/// MIC represents LoRaWAN MIC.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct MIC(pub [u8; 4]);

impl From<[u8; 4]> for MIC {
    fn from(v: [u8; 4]) -> Self {
        MIC(v)
    }
}

/// Trait for implementations of AES128 encryption.
pub trait Encrypter {
    /// Encrypt a data block.
    fn encrypt_block(&self, block: &mut GenericArray<u8, U16>);
}

/// Trait for implementations of AES128 decryption.
pub trait Decrypter {
    /// Decrypt a data block.
    fn decrypt_block(&self, block: &mut GenericArray<u8, U16>);
}

/// Trait for implementations of cipher-based message authentication code (CMAC).
pub trait Cmac {
    /// Update the data to be processed.
    fn input(&mut self, data: &[u8]);
    /// Clear the data to be processed.
    fn reset(&mut self);
    /// Process the data.
    fn result(self) -> GenericArray<u8, U16>;
}

/// Represents an abstraction over the crypto functions.
///
/// This trait provides a way to pick a different implementation of the crypto primitives.
pub trait CryptoFactory {
    /// Encrypter type.
    type E: Encrypter;
    /// Decrypter type.
    type D: Decrypter;
    /// MAC calculator type.
    type M: Cmac;

    /// Method that creates an Encrypter.
    fn new_enc(&self, key: &AES128) -> Self::E;

    /// Method that creates a Decrypter.
    fn new_dec(&self, key: &AES128) -> Self::D;

    /// Method that creates a MAC calculator.
    fn new_mac(&self, key: &AES128) -> Self::M;
}
