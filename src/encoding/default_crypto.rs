// Copyright (c) 2020 Ivaylo Petrov
//
// Licensed under the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//
// author: Ivaylo Petrov <ivajloip@gmail.com>
use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit};
use aes::Aes128;
use generic_array::typenum::U16;
use generic_array::GenericArray;

use super::keys::{self, CryptoFactory, Decrypter, Encrypter, AES128};

pub type Cmac = cmac::Cmac<Aes128>;

/// Provides a default implementation for build object for using the crypto functions.
#[derive(Default, Debug, PartialEq, Eq)]
pub struct DefaultFactory;

impl CryptoFactory for DefaultFactory {
    type E = Aes128;
    type D = Aes128;
    type M = Cmac;

    fn new_enc(&self, key: &AES128) -> Self::E {
        Aes128::new(GenericArray::from_slice(&key.0[..]))
    }

    fn new_dec(&self, key: &AES128) -> Self::D {
        Aes128::new(GenericArray::from_slice(&key.0[..]))
    }

    fn new_mac(&self, key: &AES128) -> Self::M {
        let key = GenericArray::from_slice(&key.0[..]);
        Cmac::new(key)
    }
}

impl Encrypter for Aes128 {
    fn encrypt_block(&self, block: &mut GenericArray<u8, U16>) {
        <&Self as BlockEncrypt>::encrypt_block(&self, block);
    }
}

impl Decrypter for Aes128 {
    fn decrypt_block(&self, block: &mut GenericArray<u8, U16>) {
        <&Self as BlockDecrypt>::decrypt_block(&self, block);
    }
}

impl keys::Cmac for Cmac {
    fn input(&mut self, data: &[u8]) {
        cmac::Mac::update(self, data);
    }

    fn reset(&mut self) {
        cmac::Mac::reset(self);
    }

    fn result(self) -> GenericArray<u8, U16> {
        cmac::Mac::finalize(self).into_bytes()
    }
}
