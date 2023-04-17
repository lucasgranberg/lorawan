use generic_array::GenericArray;

use crate::encoding::maccommands::{ChannelMask, Frequency};
use crate::{
    encoding::{
        keys::{CryptoFactory, Encrypter, AES128, MIC},
        maccommands::DLSettings,
        parser::*,
        securityhelpers, Error,
    },
    CfList,
};

/// DecryptedJoinAcceptPayload represents a decrypted JoinAccept.
///
/// It can be built either directly through the [new](#method.new) or using the
/// [EncryptedJoinAcceptPayload.decrypt](struct.EncryptedJoinAcceptPayload.html#method.decrypt) function.
#[derive(Debug, PartialEq, Eq)]
pub struct DecryptedJoinAcceptPayload<T, F>(T, F);

impl<T: AsRef<[u8]>, F> AsPhyPayloadBytes for DecryptedJoinAcceptPayload<T, F> {
    fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>, F: CryptoFactory> DecryptedJoinAcceptPayload<T, F> {
    pub fn new_from_encrypted(encrypted: EncryptedJoinAcceptPayload<T, F>, key: &AES128) -> Self {
        let EncryptedJoinAcceptPayload(mut bytes, factory) = encrypted;
        let len = bytes.as_ref().len();
        let aes_enc = factory.new_enc(key);

        for i in 0..(len >> 4) {
            let start = (i << 4) + 1;
            let block = GenericArray::from_mut_slice(&mut bytes.as_mut()[start..(start + 16)]);
            aes_enc.encrypt_block(block);
        }
        Self(bytes, factory)
    }
    /// Verifies that the JoinAccept has correct MIC.§
    pub fn validate_mic(&self, key: &AES128) -> bool {
        self.mic() == self.calculate_mic(key)
    }

    pub fn calculate_mic(&self, key: &AES128) -> MIC {
        let d = self.0.as_ref();
        securityhelpers::calculate_mic(&d[..d.len() - 4], self.1.new_mac(key))
    }

    /// Computes the network session key for a given device.
    ///
    /// # Argument
    ///
    /// * app_nonce - the network server nonce.
    /// * nwk_addr - the address of the network.
    /// * dev_nonce - the nonce from the device.
    /// * key - the app key.
    pub fn derive_newskey<TT: AsRef<[u8]>>(
        &self,
        dev_nonce: &DevNonce<TT>,
        key: &AES128,
    ) -> AES128 {
        self.derive_session_key(0x1, dev_nonce, key)
    }

    /// Computes the application session key for a given device.
    ///
    /// # Argument
    ///
    /// * app_nonce - the network server nonce.
    /// * nwk_addr - the address of the network.
    /// * dev_nonce - the nonce from the device.
    /// * key - the app key.
    ///
    pub fn derive_appskey<TT: AsRef<[u8]>>(
        &self,
        dev_nonce: &DevNonce<TT>,
        key: &AES128,
    ) -> AES128 {
        self.derive_session_key(0x2, dev_nonce, key)
    }

    fn derive_session_key<TT: AsRef<[u8]>>(
        &self,
        first_byte: u8,
        dev_nonce: &DevNonce<TT>,
        key: &AES128,
    ) -> AES128 {
        let cipher = self.1.new_enc(key);

        // note: AppNonce is 24 bit, NetId is 24 bit, DevNonce is 16 bit
        let app_nonce = self.app_nonce();
        let nwk_addr = self.net_id();
        let (app_nonce_arr, nwk_addr_arr, dev_nonce_arr) =
            (app_nonce.as_ref(), nwk_addr.as_ref(), dev_nonce.as_ref());

        let mut block = [0u8; 16];
        block[0] = first_byte;
        block[1] = app_nonce_arr[0];
        block[2] = app_nonce_arr[1];
        block[3] = app_nonce_arr[2];
        block[4] = nwk_addr_arr[0];
        block[5] = nwk_addr_arr[1];
        block[6] = nwk_addr_arr[2];
        block[7] = dev_nonce_arr[0];
        block[8] = dev_nonce_arr[1];

        let mut input = GenericArray::clone_from_slice(&block);
        cipher.encrypt_block(&mut input);

        let mut output_key = [0u8; 16];
        output_key.copy_from_slice(&input[0..16]);
        AES128(output_key)
    }
}
impl<T: AsRef<[u8]>, F> DecryptedJoinAcceptPayload<T, F> {
    /// Gives the app nonce of the JoinAccept.
    pub fn app_nonce(&self) -> AppNonce<&[u8]> {
        AppNonce::new_from_raw(&self.0.as_ref()[1..4])
    }

    /// Gives the net ID of the JoinAccept.
    pub fn net_id(&self) -> NwkAddr<&[u8]> {
        NwkAddr::new_from_raw(&self.0.as_ref()[4..7])
    }

    /// Gives the dev address of the JoinAccept.
    pub fn dev_addr(&self) -> DevAddr<&[u8]> {
        DevAddr::new_from_raw(&self.0.as_ref()[7..11])
    }

    /// Gives the downlink configuration of the JoinAccept.
    pub fn dl_settings(&self) -> DLSettings {
        DLSettings::new(self.0.as_ref()[11])
    }

    /// Gives the RX delay of the JoinAccept.
    pub fn rx_delay(&self) -> u8 {
        self.0.as_ref()[12] & 0x0f
    }

    /// Gives the channel frequency list of the JoinAccept.
    pub fn c_f_list(&self) -> Option<CfList> {
        if self.0.as_ref().len() == 17 {
            return None;
        }
        let d = self.0.as_ref();

        let c_f_list_type = d[28];

        if c_f_list_type == 0 {
            let res = [
                Frequency::new_from_raw(&d[13..16]),
                Frequency::new_from_raw(&d[16..19]),
                Frequency::new_from_raw(&d[19..22]),
                Frequency::new_from_raw(&d[22..25]),
                Frequency::new_from_raw(&d[25..28]),
            ];
            Some(CfList::DynamicChannel(res))
        } else if c_f_list_type == 1 {
            let res = [
                ChannelMask::new_from_raw(&d[13..15]),
                ChannelMask::new_from_raw(&d[15..17]),
                ChannelMask::new_from_raw(&d[17..19]),
                ChannelMask::new_from_raw(&d[19..21]),
                // 21..22 RFU
                // 22..25 RFU
            ];
            Some(CfList::FixedChannel(res))
        } else {
            None
        }
    }
}
/// DecryptedDataPayload represents a decrypted DataPayload.
///
/// It can be built either directly through the [new](#method.new) or using the
/// [EncryptedDataPayload.decrypt](struct.EncryptedDataPayload.html#method.decrypt) function.
#[derive(Debug, PartialEq, Eq)]
pub struct DecryptedDataPayload<T>(T);

impl<T: AsRef<[u8]>> DataHeader for DecryptedDataPayload<T> {
    fn as_data_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> DecryptedDataPayload<T> {
    pub fn new_from_encrypted<'a, F: CryptoFactory>(
        encrypted: EncryptedDataPayload<T, F>,
        nwk_skey: Option<&'a AES128>,
        app_skey: Option<&'a AES128>,
        fcnt: u32,
    ) -> Result<DecryptedDataPayload<T>, Error> {
        let fhdr_length = encrypted.fhdr_length();
        let fhdr = encrypted.fhdr();

        let full_fcnt = compute_fcnt(fcnt, fhdr.fcnt());
        let key = if encrypted.f_port().is_some() && encrypted.f_port().unwrap() != 0 {
            app_skey
        } else {
            nwk_skey
        };
        if key.is_none() {
            return Err(Error::InvalidKey);
        }
        let EncryptedDataPayload(mut data, factory) = encrypted;
        let len = data.as_ref().len();
        let start = 1 + fhdr_length + 1;
        let end = len - 4;
        if start < end {
            securityhelpers::encrypt_frm_data_payload(
                data.as_mut(),
                start,
                end,
                full_fcnt,
                &factory.new_enc(key.unwrap()),
            );
        }

        Ok(DecryptedDataPayload(data))
    }
    /// Returns FRMPayload that can represent either application payload or mac commands if fport
    /// is 0.
    pub fn frm_payload(&self) -> Result<FRMPayload, Error> {
        let data = self.as_data_bytes();
        let len = data.len();
        let fhdr_length = self.fhdr_length();
        //we have more bytes than fhdr + fport
        if len < fhdr_length + 6 {
            Ok(FRMPayload::None)
        } else if self.f_port() != Some(0) {
            // the size guarantees the existance of f_port
            Ok(FRMPayload::Data(&data[(1 + fhdr_length + 1)..(len - 4)]))
        } else {
            Ok(FRMPayload::MACCommands(FRMMacCommands::new(
                &data[(1 + fhdr_length + 1)..(len - 4)],
                self.is_uplink(),
            )))
        }
    }
}
fn compute_fcnt(old_fcnt: u32, fcnt: u16) -> u32 {
    ((old_fcnt >> 16) << 16) ^ u32::from(fcnt)
}
