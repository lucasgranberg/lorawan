use core::fmt::Debug;

use serde::{de::DeserializeOwned, Serialize};

pub trait CredentialsStore {
    #[cfg(feature = "defmt")]
    type Error: Debug + defmt::Format;

    #[cfg(not(feature = "defmt"))]
    type Error: Debug;

    fn save<C>(&mut self, credentials: &C) -> Result<(), Self::Error>
    where
        C: Sized + Serialize;

    fn load<C>(&mut self) -> Result<Option<C>, Self::Error>
    where
        C: Sized + DeserializeOwned;
}
