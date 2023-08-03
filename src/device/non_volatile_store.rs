//! Storage capability supporting persistence during power-off which must be implemented by calling code.

use core::fmt::Debug;

use crate::mac::types::Storable;

/// Specification of the functionality required of the caller for persistence.
pub trait NonVolatileStore {
    /// Possible result error.
    #[cfg(feature = "defmt")]
    type Error: Debug + defmt::Format;

    #[cfg(not(feature = "defmt"))]
    type Error: Debug;

    /// Persist data which can be converted to a u8 array.
    fn save(&mut self, storable: Storable) -> Result<(), Self::Error>;
    /// Retrieve data from persistence which can be converted from a u8 array.
    fn load(&mut self) -> Result<Storable, Self::Error>;
}
