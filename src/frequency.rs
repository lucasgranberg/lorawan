#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Frequency([u8; 3]);

#[cfg(feature = "defmt")]
impl defmt::Format for Frequency {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "Frequency({})", self.value())
    }
}

impl Frequency {
    /// Constructs a new ChannelMask from the provided data.
    pub fn new(data: &[u8]) -> Result<Self, &str> {
        if data.len() < 2 {
            return Err("not enough bytes to read");
        }
        Ok(Self::new_from_raw(data))
    }

    /// Constructs a new ChannelMask from the provided data, without verifying if they are
    /// admissible.
    ///
    /// Improper use of this method could lead to panic during runtime!
    pub fn new_from_raw(data: &[u8]) -> Self {
        let payload = [data[0], data[1], data[2]];
        Self(payload)
    }

    pub fn new_from_value(value: &u32) -> Self {
        let value = value / 100;
        let data = value.to_le_bytes();
        let payload = [data[0], data[1], data[2]];
        Self(payload)
    }

    /// Provides the decimal value in Hz of the frequency.
    pub fn value(&self) -> u32 {
        ((u32::from(self.0[2]) << 16) + (u32::from(self.0[1]) << 8) + u32::from(self.0[0])) * 100
    }
}

impl From<[u8; 3]> for Frequency {
    fn from(v: [u8; 3]) -> Self {
        Self(v)
    }
}

impl AsRef<[u8]> for Frequency {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}
