#[derive(Debug)]
pub enum Error {
    BufferFull,
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

    pub fn extend_from_slice(&mut self, buf: &[u8]) -> Result<(), Error> {
        if self.pos + buf.len() < self.packet.len() {
            self.packet[self.pos..self.pos + buf.len()].copy_from_slice(buf);
            self.pos += buf.len();
            Ok(())
        } else {
            Err(Error::BufferFull)
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
