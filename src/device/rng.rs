//! Random number generation functionality which must be implemented by calling code.

use core::fmt::Debug;
pub trait Rng {
    /// Possible result error.
    #[cfg(feature = "defmt")]
    type Error: Debug + defmt::Format;

    #[cfg(not(feature = "defmt"))]
    type Error: Debug;

    fn next_u32(&mut self) -> Result<u32, Self::Error>;
}
