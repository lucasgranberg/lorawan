#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelMask([u8; 2]);

impl ChannelMask {
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
        let payload = [data[0], data[1]];
        ChannelMask(payload)
    }

    fn channel_enabled(&self, index: usize) -> bool {
        self.0[index >> 3] & (1 << (index & 0x07)) != 0
    }

    /// Verifies if a given channel is enabled.
    pub fn is_enabled(&self, index: usize) -> Result<bool, &str> {
        if index > 15 {
            return Err("index should be between 0 and 15");
        }
        Ok(self.channel_enabled(index))
    }

    /// Provides information for each of the 16 channels if they are enabled.
    pub fn statuses(&self) -> [bool; 16] {
        let mut res = [false; 16];
        for (i, c) in res.iter_mut().enumerate() {
            *c = self.channel_enabled(i);
        }
        res
    }
}

impl From<[u8; 2]> for ChannelMask {
    fn from(v: [u8; 2]) -> Self {
        ChannelMask(v)
    }
}

impl AsRef<[u8]> for ChannelMask {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}
