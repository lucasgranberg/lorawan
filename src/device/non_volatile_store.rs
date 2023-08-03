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

    /// Save storable to persistent store.
    fn save(&mut self, storable: Storable) -> Result<(), Self::Error>;
    /// Load storable from peristent store.
    fn load(&mut self) -> Result<Storable, Self::Error>;
}
