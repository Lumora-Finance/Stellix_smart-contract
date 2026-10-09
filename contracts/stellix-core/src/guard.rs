use crate::{error::Error, storage};
use soroban_sdk::Env;

pub fn ensure_initialized(env: &Env) -> Result<(), Error> {
    if !storage::has_admin(env) {
        return Err(Error::NotInitialized);
    }
    Ok(())
}

pub fn ensure_not_paused(env: &Env) -> Result<(), Error> {
    if storage::is_paused(env) {
        return Err(Error::ContractPaused);
    }
    Ok(())
}

pub fn ensure_positive(amount: i128) -> Result<(), Error> {
    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }
    Ok(())
}
