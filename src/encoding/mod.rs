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
    UnsupportedMessageType,
    PhyDataEmpty,
    InsufficeientNumberOfBytes,
    UnsupportedMajorVersion,
    MacCommandTooBigForFOpts,
    DataAndMacCommandsInPayloadNotAllowed,
    FRMPayloadWithFportZero,
}
