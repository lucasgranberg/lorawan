//! Storage capability supporting persistence during power-off which must be implemented by calling code.

use core::fmt::Debug;

pub trait NonVolatileStore {
    #[cfg(feature = "defmt")]
    type Error: Debug + defmt::Format;

    #[cfg(not(feature = "defmt"))]
    type Error: Debug;

    fn save<'a, T>(&mut self, item: T) -> Result<(), Self::Error>
    where
        T: Sized + Into<&'a [u8]>;

    fn load<'a, T>(&'a mut self) -> Result<T, Self::Error>
    where
        T: Sized + TryFrom<&'a [u8]>;
}
