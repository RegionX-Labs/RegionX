#![cfg_attr(not(feature = "std"), no_std)]

use scale::{Decode, Encode};
use primitives::RegionId;

#[obce::definition(id = 123)]
pub trait UniquesExtension {
    fn collection_owner(&self, collection_id: RegionId) -> Result<(), UniquesError>;
}

//#[obce::error]
pub enum UniquesError {
    /// The signing account has no permission to do the operation.
    NoPermission = 1,
    /// The given item ID is unknown.
    UnknownCollection = 2,
    /// The item ID has already been used for an item.
    AlreadyExists = 3,
    /// The owner turned out to be different to what was expected.
    WrongOwner = 4,
    /// Invalid witness data given.
    BadWitness = 5,
    /// The item ID is already taken.
    InUse = 6,
    /// The item or collection is frozen.
    Frozen = 7,
    /// The delegate turned out to be different to what was expected.
    WrongDelegate = 8,
    /// There is no delegate approved.
    NoDelegate = 9,
    /// No approval exists that would allow the transfer.
    Unapproved = 10,
    /// The named owner has not signed ownership of the collection is acceptable.
    Unaccepted = 11,
    /// The item is locked.
    Locked = 12,
    /// All items have been minted.
    MaxSupplyReached = 13,
    /// The max supply has already been set.
    MaxSupplyAlreadySet = 14,
    /// The provided max supply is less than the amount of items a collection already has.
    MaxSupplyTooSmall = 15,
    /// The given item ID is unknown.
    UnknownItem = 16,
    /// Item is not for sale.
    NotForSale = 17,
    /// The provided bid is too low.
    BidTooLow = 18,
    /// Origin Caller is not supported
    OriginCannotBeCaller = 98,
    /// Unknown error
    RuntimeError = 99,
    /// Unknow status code
    UnknownStatusCode,
    /// Encountered unexpected invalid SCALE encoding
    InvalidScaleEncoding,
}

/*
impl ink::env::chain_extension::FromStatusCode for UniquesError {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::NoPermission),
            2 => Err(Self::UnknownCollection),
            3 => Err(Self::AlreadyExists),
            4 => Err(Self::WrongOwner),
            5 => Err(Self::BadWitness),
            6 => Err(Self::InUse),
            7 => Err(Self::Frozen),
            8 => Err(Self::WrongDelegate),
            9 => Err(Self::NoDelegate),
            10 => Err(Self::Unapproved),
            11 => Err(Self::Unaccepted),
            12 => Err(Self::Locked),
            13 => Err(Self::MaxSupplyReached),
            14 => Err(Self::MaxSupplyAlreadySet),
            15 => Err(Self::MaxSupplyTooSmall),
            16 => Err(Self::UnknownItem),
            17 => Err(Self::NotForSale),
            18 => Err(Self::BidTooLow),
            98 => Err(Self::OriginCannotBeCaller),
            99 => Err(Self::RuntimeError),
            _ => Err(Self::UnknownStatusCode),
        }
    }
}

impl From<scale::Error> for UniquesError {
    fn from(_: scale::Error) -> Self {
        UniquesError::InvalidScaleEncoding
    }
}

#[derive(Clone, Copy, Decode, Encode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub enum Origin {
    Caller,
    Address,
}

impl Default for Origin {
    fn default() -> Self {
        Self::Address
    }
}
*/
