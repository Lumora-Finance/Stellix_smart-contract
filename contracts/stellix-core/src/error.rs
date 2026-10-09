use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    ContractPaused = 3,
    Unauthorized = 4,
    InvalidAmount = 5,
    NotFound = 6,
    InvalidState = 7,
    InvalidInput = 8,
    Overflow = 9,
    NotDue = 10,
    InvalidSplit = 11,
    GiftExpired = 12,
    NotYetUnlocked = 13,
    NoOccurrencesRemaining = 14,
    TooManyRules = 15,
    InvalidTime = 16,
}
