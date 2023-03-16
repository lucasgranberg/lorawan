use core::fmt::Debug;
pub trait Rng {
    #[cfg(feature = "defmt")]
    type Error: Debug + defmt::Format;

    #[cfg(not(feature = "defmt"))]
    type Error: Debug;

    fn next_u32(&mut self) -> Result<u32, Self::Error>;
}
