use crate::device::Device;

pub mod default_crypto;
pub mod keys;
pub mod maccommandcreator;
pub mod maccommands;
pub mod parser;
pub mod securityhelpers;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    DataRateOutOfRange,
    TxPowerOutOfRange,
    MarginOutOfRange,
    DelayOutOfRange,
    BufferTooSmall,
    IncorrectSizeForMacCommand,
    InvalidDataRange,
    InvalidDataForJoinRequest,
    InvalidDataForEncryptedJoinAcceptPayload,
    InvalidDataForEncryptedDataPayload,
    InvalidKey,
    InvalidData,
    UnsupportedMessageType,
    PhyDataEmpty,
    InsufficeientNumberOfBytes,
    UnsupportedMajorVersion,
    MacCommandTooBigForFOpts,
    DataAndMacCommandsInPayloadNotAllowed,
    FRMPayloadWithFportZero,
}
impl<D> From<Error> for super::Error<D>
where
    D: Device,
{
    fn from(value: Error) -> Self {
        Self::Encoding(value)
    }
}
