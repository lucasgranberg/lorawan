use super::keys::{CryptoFactory, AES128, MIC};
//use super::maccommands::{parse_mac_commands, MacCommandIterator};
use super::securityhelpers;

#[cfg(feature = "default-crypto")]
use super::default_crypto::DefaultFactory;

macro_rules! fixed_len_struct {
    (
        $(#[$outer:meta])*
        struct $type:ident[$size:expr];
    ) => {
        $(#[$outer])*
        #[derive(Debug, Eq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub struct $type<T: AsRef<[u8]>>(T);

        impl<T: AsRef<[u8]>> $type<T> {
            pub fn new_from_raw(bytes: T) -> $type<T> {
                $type(bytes)
            }

            pub fn new(data: T) -> Option<$type<T>> {
                let bytes = data.as_ref();
                if bytes.len() != $size {
                    None
                } else {
                    Some($type(data))
                }
            }
        }

        impl<T: AsRef<[u8]> + Clone> Clone for $type<T> {
            fn clone(&self) -> Self {
                Self(self.0.clone())
            }
        }

        impl<T: AsRef<[u8]> + Copy> Copy for $type<T> {
        }

        impl<T: AsRef<[u8]>, V: AsRef<[u8]>> PartialEq<$type<T>> for $type<V> {
            fn eq(&self, other: &$type<T>) -> bool {
                self.as_ref() == other.as_ref()
            }
        }

        impl<'a> From<&'a [u8; $size]> for $type<&'a [u8; $size]> {
            fn from(v: &'a [u8; $size]) -> Self {
                $type(v)
            }
        }

        impl<T: AsRef<[u8]>> AsRef<[u8]> for $type<T> {
            fn as_ref(&self) -> &[u8] {
                self.0.as_ref()
            }
        }

        impl<T: AsRef<[u8]>> $type<T> {
            #[inline]
            pub fn to_owned(&self) -> $type<[u8; $size]> {
                let mut data = [0 as u8; $size];
                data.copy_from_slice(self.0.as_ref());
                $type(data)
            }
        }

        impl<T: AsRef<[u8]> + Default> Default for $type<T> {
            #[inline]
            fn default() -> $type<T> {
                $type(T::default())
            }
        }

    };
}

/// PhyPayload is a type that represents a physical LoRaWAN payload.
///
/// It can either be JoinRequest, JoinAccept, or DataPayload.
#[derive(Debug, PartialEq, Eq)]
pub enum PhyPayload<T, F> {
    JoinRequest(JoinRequestPayload<T, F>),
    JoinAccept(EncryptedJoinAcceptPayload<T, F>),
    Data(EncryptedDataPayload<T, F>),
}

impl<T: AsRef<[u8]>, F> AsRef<[u8]> for PhyPayload<T, F> {
    fn as_ref(&self) -> &[u8] {
        match self {
            PhyPayload::JoinRequest(jr) => jr.as_bytes(),
            PhyPayload::JoinAccept(ja) => ja.as_bytes(),
            PhyPayload::Data(data) => data.as_bytes(),
        }
    }
}

/// Trait with the sole purpose to make clear distinction in some implementations between types
/// that just happen to have AsRef and those that want to have the given implementations (like
/// MICAble and MHDRAble).
pub trait AsPhyPayloadBytes {
    fn as_bytes(&self) -> &[u8];
}

impl AsRef<[u8]> for dyn AsPhyPayloadBytes {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

/// Helper trait to add mic to all types that should have it.
pub trait MICAble {
    /// Gives the MIC of the PhyPayload.
    fn mic(&self) -> MIC;
}

impl<T: AsPhyPayloadBytes> MICAble for T {
    fn mic(&self) -> MIC {
        let data = self.as_bytes();
        let len = data.len();
        MIC([data[len - 4], data[len - 3], data[len - 2], data[len - 1]])
    }
}

/// Helper trait to add mhdr to all types that should have it.
pub trait MHDRAble {
    /// Gives the MIC of the PhyPayload.
    fn mhdr(&self) -> MHDR;
}

/// Assumes at least one byte in the data.
impl<T: AsPhyPayloadBytes> MHDRAble for T {
    fn mhdr(&self) -> MHDR {
        let data = self.as_bytes();
        MHDR(data[0])
    }
}

/// JoinAcceptPayload represents a JoinRequest.
///
/// It can be built either directly through the [new](#method.new) or using the
/// [parse](fn.parse.html) function.
#[derive(Debug, PartialEq, Eq)]
pub struct JoinRequestPayload<T, F>(T, F);

impl<T: AsRef<[u8]>, F> AsPhyPayloadBytes for JoinRequestPayload<T, F> {
    fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<T: AsRef<[u8]>, F: CryptoFactory> JoinRequestPayload<T, F> {
    /// Creates a new JoinRequestPayload if the provided data is acceptable.
    ///
    /// # Argument
    ///
    /// * data - the bytes for the payload.
    ///
    /// # Examples
    ///
    /// ```
    /// let data = vec![0x00, 0x04, 0x03, 0x02, 0x01, 0x04, 0x03, 0x02, 0x01, 0x05, 0x04, 0x03,
    ///     0x02, 0x05, 0x04, 0x03, 0x02, 0x2d, 0x10, 0x6a, 0x99, 0x0e, 0x12];
    /// let phy = lorawan::parser::JoinRequestPayload::new_with_factory(data,
    ///     lorawan::default_crypto::DefaultFactory);
    /// ```
    pub fn new_with_factory<'a>(data: T, factory: F) -> Result<Self, &'a str> {
        if !Self::can_build_from(data.as_ref()) {
            Err("can not build JoinRequestPayload from the provided data")
        } else {
            Ok(Self(data, factory))
        }
    }

    fn can_build_from(bytes: &[u8]) -> bool {
        bytes.len() == 23 && MHDR(bytes[0]).mtype() == MType::JoinRequest
    }

    /// Gives the APP EUI of the JoinRequest.
    pub fn app_eui(&self) -> EUI64<&[u8]> {
        EUI64::new_from_raw(&self.0.as_ref()[1..9])
    }

    /// Gives the DEV EUI of the JoinRequest.
    pub fn dev_eui(&self) -> EUI64<&[u8]> {
        EUI64::new_from_raw(&self.0.as_ref()[9..17])
    }

    /// Gives the DEV Nonce of the JoinRequest.
    pub fn dev_nonce(&self) -> DevNonce<&[u8]> {
        DevNonce::new_from_raw(&self.0.as_ref()[17..19])
    }

    /// Verifies that the JoinRequest has correct MIC.
    pub fn validate_mic(&self, key: &AES128) -> bool {
        self.mic() == self.calculate_mic(key)
    }

    fn calculate_mic(&self, key: &AES128) -> MIC {
        let d = self.0.as_ref();
        securityhelpers::calculate_mic(&d[..d.len() - 4], self.1.new_mac(key))
    }
}

/// EncryptedJoinAcceptPayload represents an encrypted JoinAccept.
///
/// It can be built either directly through the [new](#method.new) or using the
/// [parse](fn.parse.html) function.
#[derive(Debug, PartialEq, Eq)]
pub struct EncryptedJoinAcceptPayload<T, F>(pub T, pub F);

impl<T: AsRef<[u8]>, F> AsPhyPayloadBytes for EncryptedJoinAcceptPayload<T, F> {
    fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>, F: CryptoFactory> EncryptedJoinAcceptPayload<T, F> {
    /// Creates a new EncryptedJoinAcceptPayload if the provided data is acceptable.
    ///
    /// # Argument
    ///
    /// * data - the bytes for the payload.
    /// * factory - the factory that shall be used to create object for crypto functions.
    pub fn new_with_factory<'a>(data: T, factory: F) -> Result<Self, &'a str> {
        if Self::can_build_from(data.as_ref()) {
            Ok(Self(data, factory))
        } else {
            Err("can not build EncryptedJoinAcceptPayload from the provided data")
        }
    }

    fn can_build_from(bytes: &[u8]) -> bool {
        (bytes.len() == 17 || bytes.len() == 33) && MHDR(bytes[0]).mtype() == MType::JoinAccept
    }
}

/// Helper trait for EncryptedDataPayload and DecryptedDataPayload.
///
/// NOTE: Does not check the payload size as that should be done prior to building the object of
/// the implementing type.
pub trait DataHeader {
    /// Equivalent to AsRef<[u8]>.
    fn as_data_bytes(&self) -> &[u8];

    /// Gives the FHDR of the DataPayload.
    fn fhdr(&self) -> FHDR {
        FHDR::new_from_raw(
            &self.as_data_bytes()[1..(1 + self.fhdr_length())],
            self.is_uplink(),
        )
    }

    /// Gives whether the frame is confirmed
    fn is_confirmed(&self) -> bool {
        let mtype = MHDR(self.as_data_bytes()[0]).mtype();
        mtype == MType::ConfirmedDataUp || mtype == MType::ConfirmedDataDown
    }

    /// Gives whether the payload is uplink or not.
    fn is_uplink(&self) -> bool {
        let mtype = MHDR(self.as_data_bytes()[0]).mtype();
        mtype == MType::UnconfirmedDataUp || mtype == MType::ConfirmedDataUp
    }

    /// Gives the FPort of the DataPayload if there is one.
    fn f_port(&self) -> Option<u8> {
        let fhdr_length = self.fhdr_length();
        let data = self.as_data_bytes();
        if fhdr_length + 1 >= data.len() - 5 {
            return None;
        }
        Some(data[1 + fhdr_length])
    }

    /// Gives the length of the FHDR field.
    fn fhdr_length(&self) -> usize {
        fhdr_length(self.as_data_bytes()[5])
    }
}

fn fhdr_length(b: u8) -> usize {
    7 + (b & 0x0f) as usize
}

impl<T: DataHeader> AsPhyPayloadBytes for T {
    fn as_bytes(&self) -> &[u8] {
        self.as_data_bytes()
    }
}

/// EncryptedDataPayload represents an encrypted data payload.
///
/// It can be built either directly through the [new](#method.new) or using the
/// [parse](fn.parse.html) function.
#[derive(Debug, PartialEq, Eq)]
pub struct EncryptedDataPayload<T, F>(pub T, pub F);

impl<T: AsRef<[u8]>, F> DataHeader for EncryptedDataPayload<T, F> {
    fn as_data_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<T: AsRef<[u8]>, F: CryptoFactory> EncryptedDataPayload<T, F> {
    /// Creates a new EncryptedDataPayload if the provided data is acceptable.
    ///
    /// # Argument
    ///
    /// * data - the bytes for the payload.
    /// * factory - the factory that shall be used to create object for crypto functions.
    pub fn new_with_factory<'a>(data: T, factory: F) -> Result<Self, &'a str> {
        if Self::can_build_from(data.as_ref()) {
            Ok(Self(data, factory))
        } else {
            Err("can not build EncryptedDataPayload from the provided data")
        }
    }

    fn can_build_from(bytes: &[u8]) -> bool {
        let has_acceptable_len = bytes.len() >= 12 &&
            // TODO: Bug related to possibly insufficient number of bytes
            fhdr_length(bytes[5]) <= bytes.len();
        if !has_acceptable_len {
            return false;
        }

        matches!(
            MHDR(bytes[0]).mtype(),
            MType::ConfirmedDataUp
                | MType::ConfirmedDataDown
                | MType::UnconfirmedDataUp
                | MType::UnconfirmedDataDown
        )
    }

    /// Verifies that the DataPayload has correct MIC.
    pub fn validate_mic(&self, key: &AES128, fcnt: u32) -> bool {
        self.mic() == self.calculate_mic(key, fcnt)
    }

    fn calculate_mic(&self, key: &AES128, fcnt: u32) -> MIC {
        let d = self.0.as_ref();
        securityhelpers::calculate_data_mic(&d[..d.len() - 4], self.1.new_mac(key), fcnt)
    }
}

fn compute_fcnt(old_fcnt: u32, fcnt: u16) -> u32 {
    ((old_fcnt >> 16) << 16) ^ u32::from(fcnt)
}

/// Parses a payload as LoRaWAN physical payload.
///
/// # Argument
///
/// * bytes - the data from which the PhyPayload is to be built.
///
/// # Examples
///
/// ```
/// let mut data = vec![0x40, 0x04, 0x03, 0x02, 0x01, 0x80, 0x01, 0x00, 0x01,
///     0xa6, 0x94, 0x64, 0x26, 0x15, 0xd6, 0xc3, 0xb5, 0x82];
/// if let Ok(lorawan::parser::PhyPayload::Data(phy)) = lorawan::parser::parse(data) {
///     println!("{:?}", phy);
/// } else {
///     panic!("failed to parse data payload");
/// }
/// ```
#[cfg(feature = "default-crypto")]
pub fn parse<'a, T: AsRef<[u8]> + AsMut<[u8]>>(
    data: T,
) -> Result<PhyPayload<T, DefaultFactory>, &'a str> {
    parse_with_factory(data, DefaultFactory)
}

/// Parses a payload as LoRaWAN physical payload.
///
/// Check out [parse](fn.parse.html) if you do not need custom crypto factory.  
///
/// Returns error "Unsupported major version" if the major version is unsupported.
///
/// # Argument
///
/// * bytes - the data from which the PhyPayload is to be built.
/// * factory - the factory that shall be used to create object for crypto functions.
pub fn parse_with_factory<'a, T, F>(data: T, factory: F) -> Result<PhyPayload<T, F>, &'static str>
where
    T: AsRef<[u8]> + AsMut<[u8]>,
    F: CryptoFactory,
{
    let bytes = data.as_ref();
    check_phy_data(bytes)?;
    match MHDR(bytes[0]).mtype() {
        MType::JoinRequest => Ok(PhyPayload::JoinRequest(
            JoinRequestPayload::new_with_factory(data, factory)?,
        )),
        MType::JoinAccept => Ok(PhyPayload::JoinAccept(
            EncryptedJoinAcceptPayload::new_with_factory(data, factory)?,
        )),
        MType::UnconfirmedDataUp
        | MType::ConfirmedDataUp
        | MType::UnconfirmedDataDown
        | MType::ConfirmedDataDown => Ok(PhyPayload::Data(EncryptedDataPayload::new_with_factory(
            data, factory,
        )?)),
        _ => Err("unsupported message type"),
    }
}

fn check_phy_data(bytes: &[u8]) -> Result<(), &'static str> {
    let len = bytes.len();
    if len == 0 {
        return Err("can not parse empty payload as LoRaWAN phy payload");
    }
    let mhdr = MHDR(bytes[0]);
    if mhdr.major() != Major::LoRaWANR1 {
        return Err("Unsupported major version");
    }
    // the smallest payload is a data payload without fport and FRMPayload
    // which is 12 bytes long.
    if len < 12 {
        Err("insufficient number of bytes")
    } else {
        Ok(())
    }
}

/// MHDR represents LoRaWAN MHDR.
#[derive(Debug, PartialEq, Eq)]
pub struct MHDR(u8);

impl MHDR {
    pub fn new(byte: u8) -> MHDR {
        MHDR(byte)
    }

    /// Gives the type of message that PhyPayload is carrying.
    pub fn mtype(&self) -> MType {
        match self.0 >> 5 {
            0 => MType::JoinRequest,
            1 => MType::JoinAccept,
            2 => MType::UnconfirmedDataUp,
            3 => MType::UnconfirmedDataDown,
            4 => MType::ConfirmedDataUp,
            5 => MType::ConfirmedDataDown,
            6 => MType::RFU,
            _ => MType::Proprietary,
        }
    }

    /// Gives the version of LoRaWAN payload format.
    pub fn major(&self) -> Major {
        if self.0.trailing_zeros() >= 2 {
            Major::LoRaWANR1
        } else {
            Major::RFU
        }
    }
}

impl From<u8> for MHDR {
    fn from(v: u8) -> Self {
        MHDR(v)
    }
}

/// MType gives the possible message types of the PhyPayload.
#[derive(Debug, PartialEq, Eq)]
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

/// Major gives the supported LoRaWAN payload formats.
#[derive(Debug, PartialEq, Eq)]
pub enum Major {
    LoRaWANR1,
    RFU,
}

fixed_len_struct! {
    /// EUI64 represents a 64 bit EUI.
    struct EUI64[8];
}

fixed_len_struct! {
    /// DevNonce represents a 16 bit device nonce.
    struct DevNonce[2];
}

fixed_len_struct! {
    /// AppNonce represents a 24 bit network server nonce.
    struct AppNonce[3];
}

fixed_len_struct! {
    /// DevAddr represents a 32 bit device address.
    struct DevAddr[4];
}

#[allow(clippy::should_implement_trait)]
impl<T: AsRef<[u8]>> DevAddr<T> {
    pub fn nwk_id(&self) -> u8 {
        self.0.as_ref()[0] >> 1
    }
    pub fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

fixed_len_struct! {
    /// NwkAddr represents a 24 bit network address.
    struct NwkAddr[3];
}

/// FHDR represents FHDR from DataPayload.
#[derive(Debug, PartialEq, Eq)]
pub struct FHDR<'a>(pub(crate) &'a [u8], pub(crate) bool);

impl<'a> FHDR<'a> {
    pub fn new_from_raw(bytes: &'a [u8], uplink: bool) -> FHDR {
        FHDR(bytes, uplink)
    }

    pub fn new(bytes: &'a [u8], uplink: bool) -> Option<FHDR> {
        let data_len = bytes.len();
        if data_len < 7 {
            return None;
        }
        if data_len < fhdr_length(bytes[4]) {
            return None;
        }
        Some(FHDR(bytes, uplink))
    }

    /// Gives the device address associated with the given payload.
    pub fn dev_addr(&self) -> DevAddr<&'a [u8]> {
        DevAddr::new_from_raw(&self.0[0..4])
    }

    /// Gives the FCtrl associated with the given payload.
    pub fn fctrl(&self) -> FCtrl {
        FCtrl(self.0[4], self.1)
    }

    /// Gives the truncated FCnt associated with the given payload.
    pub fn fcnt(&self) -> u16 {
        (u16::from(self.0[6]) << 8) | u16::from(self.0[5])
    }

    pub fn fopts_len(&self) -> u8 {
        FCtrl(self.0[4], self.1).f_opts_len()
    }
}

/// FCtrl represents the FCtrl from FHDR.
#[derive(Debug, PartialEq, Eq)]
pub struct FCtrl(pub u8, pub bool);

impl FCtrl {
    pub fn set_ack(&mut self) {
        self.0 |= 0b1 << 5;
    }

    pub fn new(bytes: u8, uplink: bool) -> FCtrl {
        FCtrl(bytes, uplink)
    }

    /// Gives whether ADR is enabled or not.
    pub fn adr(&self) -> bool {
        self.0 >> 7 == 1
    }

    /// Gives whether ADR ACK is requested.
    pub fn adr_ack_req(&self) -> bool {
        self.1 && self.0 & (1 << 6) != 0
    }

    /// Gives whether ack bit is set.
    pub fn ack(&self) -> bool {
        self.0 & (1 << 5) != 0
    }

    /// Gives whether there are more payloads pending.
    pub fn f_pending(&self) -> bool {
        !self.1 && self.0 & (1 << 4) != 0
    }

    /// Gives the size of FOpts.
    pub fn f_opts_len(&self) -> u8 {
        self.0 & 0x0f
    }

    /// Gives the binary representation of the FCtrl.
    pub fn raw_value(&self) -> u8 {
        self.0
    }
}

/// FRMPayload represents the FRMPayload that can either be the application
/// data or mac commands.
#[derive(Debug, PartialEq, Eq)]
pub enum FRMPayload<'a> {
    Data(&'a [u8]),
    MACCommands(FRMMacCommands<'a>),
    None,
}

/// FRMMacCommands represents the mac commands.
#[derive(Debug, PartialEq, Eq)]
pub struct FRMMacCommands<'a>(pub(crate) bool, pub(crate) &'a [u8]);

impl<'a> FRMMacCommands<'a> {
    pub fn new(bytes: &'a [u8], uplink: bool) -> Self {
        FRMMacCommands(uplink, bytes)
    }
}
