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
#[derive(Copy, Clone, Debug)]
pub struct RxQuality {
    rssi: i16,
    snr: i8,
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

pub struct RadioBuffer<const N: usize> {
    packet: [u8; N],
    pos: usize,
}
impl<const N: usize> Default for RadioBuffer<N> {
    fn default() -> Self {
        Self {
            packet: [0; N],
            pos: Default::default(),
        }
    }
}

impl<const N: usize> RadioBuffer<N> {
    pub fn new() -> Self {
        Self {
            packet: [0; N],
            pos: 0,
        }
    }

    pub fn clear(&mut self) {
        self.pos = 0;
    }

    pub fn extend_from_slice(&mut self, buf: &[u8]) -> Result<(), ()> {
        if self.pos + buf.len() < self.packet.len() {
            self.packet[self.pos..self.pos + buf.len()].copy_from_slice(buf);
            self.pos += buf.len();
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn as_raw_slice(&mut self) -> &mut [u8] {
        &mut self.packet
    }

    pub fn inc(&mut self, len: usize) {
        assert!(self.pos + len < self.packet.len());
        self.pos += len;
    }
}

impl<const N: usize> AsMut<[u8]> for RadioBuffer<N> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.packet[..self.pos]
    }
}

impl<const N: usize> AsRef<[u8]> for RadioBuffer<N> {
    fn as_ref(&self) -> &[u8] {
        &self.packet[..self.pos]
    }
}
