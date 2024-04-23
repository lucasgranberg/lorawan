//! Types used to control communication with the LoRa physical layer.

use lora_modulation::{Bandwidth, CodingRate, SpreadingFactor};

/// LoRaWAN radio signal configuration.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug)]
pub struct RfConfig {
    /// Frequency in Hz.
    pub frequency: u32,
    /// Coding rate (ratio of actual data bits to error-correcting data bits).
    pub coding_rate: CodingRate,
    /// Data rate (bandwidth and spreading factor).
    pub data_rate: Datarate,
}

/// LoRaWAN data rate.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone)]
pub struct Datarate {
    /// Bandwidth.
    pub bandwidth: Bandwidth,
    /// Spreading factor.
    pub spreading_factor: SpreadingFactor,
}

/// LoRaWAN packet transmission configuration.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug)]
pub struct TxConfig {
    /// Power.
    pub pw: i8,
    /// Radio signal configuration.
    pub rf: RfConfig,
}

/// LoRaWAN packet reception quality.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub struct RxQuality {
    /// Received signal strength indication.
    pub rssi: i16,
    /// Signal-to-noise ratio.
    pub snr: i8,
}

impl RxQuality {
    /// Creation.
    pub fn new(rssi: i16, snr: i8) -> RxQuality {
        RxQuality { rssi, snr }
    }

    /// Get the RSSI property.
    pub fn rssi(self) -> i16 {
        self.rssi
    }

    /// Get the SNR property.
    pub fn snr(self) -> i8 {
        self.snr
    }
}
