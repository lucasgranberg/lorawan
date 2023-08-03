//! Buffer functionality for send/receive data tranmission between the caller and the LoRa physical layer.

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(missing_docs)]
pub enum Error {
    BufferFull,
}

/// Packet buffer used to send/receive data.
#[derive(Clone)]
pub struct RadioBuffer<const N: usize> {
    packet: [u8; N],
    pos: usize,
}
impl<const N: usize> Default for RadioBuffer<N> {
    fn default() -> Self {
        Self { packet: [0; N], pos: Default::default() }
    }
}

impl<const N: usize> RadioBuffer<N> {
    /// Creation.
    pub fn new() -> Self {
        Self { packet: [0; N], pos: 0 }
    }

    /// Mark as empty.
    pub fn clear(&mut self) {
        self.pos = 0;
    }

    /// Add data as long as it fits within the buffer.
    pub fn extend_from_slice(&mut self, buf: &[u8]) -> Result<(), Error> {
        if self.pos + buf.len() < self.packet.len() {
            self.packet[self.pos..self.pos + buf.len()].copy_from_slice(buf);
            self.pos += buf.len();
            Ok(())
        } else {
            Err(Error::BufferFull)
        }
    }

    /// Provide the mutable buffer without regard to contained data.
    pub fn as_raw_slice(&mut self) -> &mut [u8] {
        &mut self.packet
    }

    /// Update the start position for subsequent data additions.
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
