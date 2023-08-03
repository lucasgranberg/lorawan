//! Random number generation functionality which must be implemented by calling code.

use core::fmt::Debug;

/// Specification of the functionality required of the caller for random number generation.
pub trait Rng {
    /// Possible result error.
    #[cfg(feature = "defmt")]
    type Error: Debug + defmt::Format;

    #[cfg(not(feature = "defmt"))]
    type Error: Debug;

    /// Get the next random number.
    fn next_u32(&mut self) -> Result<u32, Self::Error>;
}
