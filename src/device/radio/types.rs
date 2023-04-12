#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone)]
pub enum Bandwidth {
    _125KHz,
    _250KHz,
    _500KHz,
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

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone)]
pub enum CodingRate {
    _4_5,
    _4_6,
    _4_7,
    _4_8,
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
    pub pw: i8,
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
