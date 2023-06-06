use lora_phy;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone)]
pub enum Bandwidth {
    _125KHz,
    _250KHz,
    _500KHz,
}

/// Convert the bandwidth for use in the external lora-phy crate
impl From<Bandwidth> for lora_phy::mod_params::Bandwidth {
    fn from(bw: Bandwidth) -> Self {
        match bw {
            Bandwidth::_125KHz => lora_phy::mod_params::Bandwidth::_125KHz,
            Bandwidth::_250KHz => lora_phy::mod_params::Bandwidth::_250KHz,
            Bandwidth::_500KHz => lora_phy::mod_params::Bandwidth::_500KHz,
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone)]
pub enum SpreadingFactor {
    _7,
    _8,
    _9,
    _10,
    _11,
    _12,
}

/// Convert the spreading factor for use in the external lora-phy crate
impl From<SpreadingFactor> for lora_phy::mod_params::SpreadingFactor {
    fn from(sf: SpreadingFactor) -> Self {
        match sf {
            SpreadingFactor::_7 => lora_phy::mod_params::SpreadingFactor::_7,
            SpreadingFactor::_8 => lora_phy::mod_params::SpreadingFactor::_8,
            SpreadingFactor::_9 => lora_phy::mod_params::SpreadingFactor::_9,
            SpreadingFactor::_10 => lora_phy::mod_params::SpreadingFactor::_10,
            SpreadingFactor::_11 => lora_phy::mod_params::SpreadingFactor::_11,
            SpreadingFactor::_12 => lora_phy::mod_params::SpreadingFactor::_12,
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone)]
pub enum CodingRate {
    _4_5,
    _4_6,
    _4_7,
    _4_8,
}

/// Convert the coding rate for use in the external lora-phy crate
impl From<CodingRate> for lora_phy::mod_params::CodingRate {
    fn from(cr: CodingRate) -> Self {
        match cr {
            CodingRate::_4_5 => lora_phy::mod_params::CodingRate::_4_5,
            CodingRate::_4_6 => lora_phy::mod_params::CodingRate::_4_6,
            CodingRate::_4_7 => lora_phy::mod_params::CodingRate::_4_7,
            CodingRate::_4_8 => lora_phy::mod_params::CodingRate::_4_8,
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug)]
pub struct RfConfig {
    pub frequency: u32,
    pub coding_rate: CodingRate,
    pub data_rate: Datarate,
}
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone)]
pub struct Datarate {
    pub bandwidth: Bandwidth,
    pub spreading_factor: SpreadingFactor,
}
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug)]
pub struct TxConfig {
    pub pw: u8,
    pub rf: RfConfig,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub struct RxQuality {
    pub rssi: i16,
    pub snr: i8,
}

impl RxQuality {
    pub fn new(rssi: i16, snr: i8) -> RxQuality {
        RxQuality { rssi, snr }
    }

    pub fn rssi(self) -> i16 {
        self.rssi
    }
    pub fn snr(self) -> i8 {
        self.snr
    }
}
