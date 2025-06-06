//! Properties used in LoRaWAN MAC processing.

use encoding::default_crypto::DefaultFactory;
use encoding::keys::{AppEui, AppKey, AppSKey, DevEui, NwkSKey};
use encoding::parser::{DecryptedJoinAcceptPayload, DevAddr, DevNonce};

pub(crate) struct RxWindows {
    pub(crate) rx1_open: u16,
    pub(crate) rx2_open: u16,
}
impl RxWindows {
    pub(crate) fn get_open(&self, window: &Window) -> u16 {
        match window {
            Window::_1 => self.rx1_open,
            Window::_2 => self.rx2_open,
        }
    }
}

/// Basic send/receive properties.
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Configuration {
    pub(crate) max_duty_cycle: f32,
    pub(crate) tx_power: Option<i8>,
    pub(crate) tx_data_rate: Option<DR>,
    pub(crate) rx1_data_rate_offset: Option<u8>,
    pub(crate) rx_delay: Option<u8>,
    pub(crate) rx2_data_rate: Option<DR>,
    pub(crate) rx2_frequency: Option<u32>,
    pub(crate) number_of_transmissions: u8,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            tx_data_rate: None,
            tx_power: None,
            max_duty_cycle: 0.0,
            rx1_data_rate_offset: None,
            rx_delay: None,
            rx2_data_rate: None,
            rx2_frequency: None,
            number_of_transmissions: 1,
        }
    }
}

/// Identification properties used to enable communication with a network server.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Credentials {
    pub(crate) app_eui: AppEui,
    pub(crate) dev_eui: DevEui,
    pub(crate) app_key: AppKey,
    pub(crate) dev_nonce: u16,
}
impl Credentials {
    /// Creation.
    pub fn new(app_eui: [u8; 8], dev_eui: [u8; 8], app_key: [u8; 16]) -> Self {
        Self {
            app_eui: app_eui.into(),
            dev_eui: dev_eui.into(),
            app_key: app_key.into(),
            dev_nonce: 0,
        }
    }

    /// Increment the nonce associated with a join request.
    pub fn incr_dev_nonce(&mut self) {
        self.dev_nonce += 1;
    }
}

/// Properties maintained during a session with a network server.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Session {
    pub(crate) nwkskey: NwkSKey,
    pub(crate) appskey: AppSKey,
    pub(crate) devaddr: DevAddr<[u8; 4]>,
    pub(crate) fcnt_up: u32,
    pub(crate) fcnt_down: u32,
    pub(crate) adr_ack_cnt: u8,
}
impl Session {
    /// Creation.
    pub fn derive_new<T: AsRef<[u8]> + AsMut<[u8]>>(
        decrypt: &DecryptedJoinAcceptPayload<T>,
        devnonce: DevNonce<[u8; 2]>,
        credentials: &Credentials,
    ) -> Self {
        Self::new(
            decrypt.derive_nwkskey(&devnonce, &credentials.app_key, &DefaultFactory),
            decrypt.derive_appskey(&devnonce, &credentials.app_key, &DefaultFactory),
            DevAddr::new([
                decrypt.dev_addr().as_ref()[0],
                decrypt.dev_addr().as_ref()[1],
                decrypt.dev_addr().as_ref()[2],
                decrypt.dev_addr().as_ref()[3],
            ])
            .unwrap(),
        )
    }

    /// Creation.
    pub fn new(nwkskey: NwkSKey, appskey: AppSKey, devaddr: DevAddr<[u8; 4]>) -> Self {
        Self { nwkskey, appskey, devaddr, fcnt_up: 0, fcnt_down: 0, adr_ack_cnt: 0 }
    }

    /// Get the network session key.
    pub fn nwkskey(&self) -> &NwkSKey {
        &self.nwkskey
    }

    /// Get the application session key.
    pub fn appskey(&self) -> &AppSKey {
        &self.appskey
    }

    /// Get the device address.
    pub fn devaddr(&self) -> &DevAddr<[u8; 4]> {
        &self.devaddr
    }

    /// Increment the uplink frame count.
    pub fn fcnt_up_increment(&mut self) {
        self.fcnt_up += 1;
    }

    /// Has the uplink frame count reached or exceeded the limit?
    pub fn is_expired(&self) -> bool {
        self.fcnt_up >= 0xFFFF
    }
    /// clear adr ack count
    pub fn adr_ack_cnt_clear(&mut self) {
        self.adr_ack_cnt = 0;
    }

    /// increment adr ack count
    pub fn adr_ack_cnt_increment(&mut self) {
        if let Some(val) = self.adr_ack_cnt.checked_add(1) {
            self.adr_ack_cnt = val
        };
    }
}

/// Basic send/receive properties persisted in non-volatile storage for
/// continuity across power-on cycles.
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[allow(missing_docs)]
pub struct Storable {
    pub rx1_data_rate_offset: Option<u8>,
    pub rx_delay: Option<u8>,
    pub rx2_data_rate: Option<DR>,
    pub rx2_frequency: Option<u32>,
    pub dev_nonce: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[allow(missing_docs)]
#[repr(u8)]
pub enum DR {
    _0 = 0,
    _1 = 1,
    _2 = 2,
    _3 = 3,
    _4 = 4,
    _5 = 5,
    _6 = 6,
    _7 = 7,
    _8 = 8,
    _9 = 9,
    _10 = 10,
    _11 = 11,
    _12 = 12,
    _13 = 13,
    _14 = 14,
    _15 = 15,
}

impl DR {
    /// Is this DR within range?
    pub fn in_range(&self, range: (DR, DR)) -> bool {
        (range.0 as u8 <= *self as u8) && (*self as u8 <= range.1 as u8)
    }
}

impl TryFrom<u8> for DR {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(DR::_0),
            1 => Ok(DR::_1),
            2 => Ok(DR::_2),
            3 => Ok(DR::_3),
            4 => Ok(DR::_4),
            5 => Ok(DR::_5),
            6 => Ok(DR::_6),
            7 => Ok(DR::_7),
            8 => Ok(DR::_8),
            9 => Ok(DR::_9),
            10 => Ok(DR::_10),
            11 => Ok(DR::_11),
            12 => Ok(DR::_12),
            13 => Ok(DR::_13),
            14 => Ok(DR::_14),
            15 => Ok(DR::_15),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum MType {
    JoinRequest,
    JoinAccept,
    UnconfirmedDataUp,
    UnconfirmedDataDown,
    ConfirmedDataUp,
    ConfirmedDataDown,
    RFU,
    Proprietary,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Frame {
    Join,
    Data,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Window {
    _1,
    _2,
}
