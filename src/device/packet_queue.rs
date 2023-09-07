//! Packet queue, implemented by calling code, used to pass uplink and downlink messages between the end device app
//! and the LoRaWAN task.

use core::fmt::Debug;

use super::packet_buffer::PacketBuffer;

/// Maximum length of a packet managed by a PacketQueue.
pub const PACKET_SIZE: usize = 256;

/// Use a queue concept for decoupling of packet handling within this network protocol, which has with tight receive window constraints.
/// The underlying implementation can use whatever feature makes sense for the embedded framework, such as pubsub between
/// the end user device app task and the LoRaWAN task.
pub trait PacketQueue: Sized {
    /// Possible result error.
    #[cfg(feature = "defmt")]
    type Error: Debug + defmt::Format;
    #[cfg(not(feature = "defmt"))]
    type Error: Debug;

    /// Push a packet onto the queue.
    async fn push(&mut self, packet: PacketBuffer<PACKET_SIZE>) -> Result<(), Self::Error>;
    /// Get the oldest packet from the queue.
    async fn next(&mut self) -> Result<PacketBuffer<PACKET_SIZE>, Self::Error>;
    /// Determine if the queue holds one or more packets.
    fn available(&mut self) -> bool;
}
