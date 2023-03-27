// Copyright (c) 2017-2020 Ivaylo Petrov
//
// Licensed under the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//
// author: Ivaylo Petrov <ivajloip@gmail.com>

//! Provides types and methods for creating LoRaWAN payloads.
//!
//! See [JoinAcceptCreator.new](struct.JoinAcceptCreator.html#method.new) for an example.

use crate::encoding::{
    keys::{self, CryptoFactory},
    maccommandcreator::build_mac_commands,
    maccommands::{mac_commands_len, SerializableMacCommand},
    parser::{DevAddr, DevNonce, FCtrl, EUI64},
    securityhelpers, Error,
};
const PIGGYBACK_MAC_COMMANDS_MAX_LEN: usize = 15;

fn set_mic<F: CryptoFactory>(data: &mut [u8], key: &keys::AES128, factory: &F) {
    let len = data.len();
    let mic = securityhelpers::calculate_mic(&data[..len - 4], factory.new_mac(key));

    data[len - 4..].copy_from_slice(&mic.0[..]);
}

/// JoinRequestCreator serves for creating binary representation of Physical
/// Payload of JoinRequest.
#[derive(Default)]
pub struct JoinRequestCreator<D, F> {
    data: D,
    factory: F,
}

impl<D: AsMut<[u8]>, F: CryptoFactory> JoinRequestCreator<D, F> {
    /// Sets the application EUI of the JoinRequest to the provided value.
    ///
    /// # Argument
    ///
    /// * app_eui - instance of lorawan::encoding::parser::EUI64 or anything that can
    ///   be converted into it.
    pub fn set_app_eui<H: AsRef<[u8]>, T: Into<EUI64<H>>>(&mut self, app_eui: T) -> &mut Self {
        let converted = app_eui.into();
        self.data.as_mut()[1..9].copy_from_slice(converted.as_ref());

        self
    }

    /// Sets the device EUI of the JoinRequest to the provided value.
    ///
    /// # Argument
    ///
    /// * dev_eui - instance of lorawan::encoding::parser::EUI64 or anything that can
    ///   be converted into it.
    pub fn set_dev_eui<H: AsRef<[u8]>, T: Into<EUI64<H>>>(&mut self, dev_eui: T) -> &mut Self {
        let converted = dev_eui.into();
        self.data.as_mut()[9..17].copy_from_slice(converted.as_ref());

        self
    }

    /// Sets the device nonce of the JoinRequest to the provided value.
    ///
    /// # Argument
    ///
    /// * dev_nonce - instance of lorawan::encoding::parser::DevNonce or anything that can
    ///   be converted into it.
    pub fn set_dev_nonce<H: AsRef<[u8]>, T: Into<DevNonce<H>>>(
        &mut self,
        dev_nonce: T,
    ) -> &mut Self {
        let converted = dev_nonce.into();
        self.data.as_mut()[17..19].copy_from_slice(converted.as_ref());

        self
    }

    /// Provides the binary representation of the JoinRequest physical payload
    /// with the MIC set.
    ///
    /// # Argument
    ///
    /// * key - the key to be used for setting the MIC.
    pub fn build(&mut self, key: &keys::AES128) -> Result<&[u8], Error> {
        let d = self.data.as_mut();
        set_mic(d, key, &self.factory);
        Ok(d)
    }
}

/// DataPayloadCreator serves for creating binary representation of Physical
/// Payload of DataUp or DataDown messages.
pub struct DataPayloadCreator<D, F> {
    data: D,
    data_f_port: Option<u8>,
    fcnt: u32,
    factory: F,
}
#[allow(dead_code)]
impl<D: AsMut<[u8]>, F: CryptoFactory> DataPayloadCreator<D, F> {
    pub fn new(data: D, factory: F) -> Self {
        DataPayloadCreator {
            data,
            data_f_port: None,
            fcnt: 0,
            factory,
        }
    }

    /// Sets whether the packet is uplink or downlink.
    ///
    /// # Argument
    ///
    /// * uplink - whether the packet is uplink or downlink.
    pub fn set_uplink(&mut self, uplink: bool) -> &mut Self {
        if uplink {
            self.data.as_mut()[0] &= 0xdf;
        } else {
            self.data.as_mut()[0] |= 0x20;
        }
        self
    }

    /// Sets whether the packet is confirmed or unconfirmed.
    ///
    /// # Argument
    ///
    /// * confirmed - whether the packet is confirmed or unconfirmed.
    pub fn set_confirmed(&mut self, confirmed: bool) -> &mut Self {
        let d = self.data.as_mut();
        if confirmed {
            d[0] &= 0xbf;
            d[0] |= 0x80;
        } else {
            d[0] &= 0x7f;
            d[0] |= 0x40;
        }

        self
    }

    /// Sets the device address of the DataPayload to the provided value.
    ///
    /// # Argument
    ///
    /// * dev_addr - instance of lorawan::encoding::parser::DevAddr or anything that can
    ///   be converted into it.
    pub fn set_dev_addr<H: AsRef<[u8]>, T: Into<DevAddr<H>>>(&mut self, dev_addr: T) -> &mut Self {
        let converted = dev_addr.into();
        self.data.as_mut()[1..5].copy_from_slice(converted.as_ref());

        self
    }

    /// Sets the FCtrl header of the DataPayload packet to the specified value.
    ///
    /// # Argument
    ///
    /// * fctrl - the FCtrl to be set.
    pub fn set_fctrl(&mut self, fctrl: &FCtrl) -> &mut Self {
        self.data.as_mut()[5] = fctrl.raw_value();
        self
    }

    /// Sets the FCnt header of the DataPayload packet to the specified value.
    ///
    /// NOTE: In the packet header the value will be truncated to u16.
    ///
    /// # Argument
    ///
    /// * fctrl - the FCtrl to be set.
    pub fn set_fcnt(&mut self, fcnt: u32) -> &mut Self {
        let d = self.data.as_mut();
        self.fcnt = fcnt;
        d[6] = (fcnt & (0xff_u32)) as u8;
        d[7] = (fcnt >> 8) as u8;

        self
    }

    /// Sets the FPort header of the DataPayload packet to the specified value.
    ///
    /// If f_port == 0, automatically sets `encrypt_mac_commands` to `true`.
    ///
    /// # Argument
    ///
    /// * f_port - the FPort to be set.
    pub fn set_f_port(&mut self, f_port: u8) -> &mut Self {
        self.data_f_port = Some(f_port);

        self
    }

    /// Whether a set of mac commands can be piggybacked.
    pub fn can_piggyback(cmds: &[&dyn SerializableMacCommand]) -> bool {
        mac_commands_len(cmds) <= PIGGYBACK_MAC_COMMANDS_MAX_LEN
    }

    /// Provides the binary representation of the DataPayload physical payload
    /// with the MIC set and payload encrypted.
    ///
    /// # Argument
    ///
    /// * payload - the FRMPayload (application) to be sent.
    /// * nwk_skey - the key to be used for setting the MIC and possibly for
    ///   MAC command encryption.
    /// * app_skey - the key to be used for payload encryption if fport not 0,
    ///   otherwise nwk_skey is only used.
    pub fn build(
        &mut self,
        payload: &[u8],
        cmds: &[&dyn SerializableMacCommand],
        nwk_skey: &keys::AES128,
        app_skey: &keys::AES128,
    ) -> Result<&[u8], Error> {
        let d = self.data.as_mut();
        let mut last_filled = 8; // MHDR + FHDR without the FOpts
        let has_fport = self.data_f_port.is_some();
        let has_fport_zero = has_fport && self.data_f_port.unwrap() == 0;
        let mac_cmds_len = mac_commands_len(cmds);

        // Set MAC Commands
        if mac_cmds_len > PIGGYBACK_MAC_COMMANDS_MAX_LEN && !has_fport_zero {
            return Err(Error::MacCommandTooBigForFOpts);
        }

        // Set FPort
        let mut payload_len = payload.len();
        if has_fport_zero && payload_len > 0 {
            return Err(Error::DataAndMacCommandsInPayloadNotAllowed);
        }
        if !has_fport && payload_len > 0 {
            return Err(Error::FRMPayloadWithFportZero);
        }
        // Set FOptsLen if present
        if !has_fport_zero && mac_cmds_len > 0 {
            d[5] |= mac_cmds_len as u8 & 0x0f;
            build_mac_commands(cmds, &mut d[last_filled..last_filled + mac_cmds_len]).unwrap();
            last_filled += mac_cmds_len;
        }
        if has_fport {
            d[last_filled] = self.data_f_port.unwrap();
            last_filled += 1;
        }

        let mut enc_key = app_skey;
        if mac_cmds_len > 0 && has_fport_zero {
            enc_key = nwk_skey;
            payload_len = mac_cmds_len;
            build_mac_commands(cmds, &mut d[last_filled..last_filled + payload_len]).unwrap();
        } else {
            d[last_filled..last_filled + payload_len].copy_from_slice(payload);
        };

        // Encrypt FRMPayload
        securityhelpers::encrypt_frm_data_payload(
            d,
            last_filled,
            last_filled + payload_len,
            self.fcnt,
            &self.factory.new_enc(enc_key),
        );

        // MIC set
        let mic = securityhelpers::calculate_data_mic(
            &d[..last_filled + payload_len],
            self.factory.new_mac(nwk_skey),
            self.fcnt,
        );
        d[last_filled + payload_len..last_filled + payload_len + 4].copy_from_slice(&mic.0);

        Ok(&d[..last_filled + payload_len + 4])
    }
}
