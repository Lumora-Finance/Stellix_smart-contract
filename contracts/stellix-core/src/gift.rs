use crate::{
    error::Error,
    events::{GiftCancelled, GiftClaimed, GiftCreated},
    guard, storage,
    types::{Gift, GiftStatus},
};
use soroban_sdk::{token::TokenClient, Address, Env, String};

#[allow(clippy::too_many_arguments)]
pub fn create_gift(
    env: &Env,
    sender: Address,
    recipient: Address,
    token: Address,
    amount: i128,
    message: String,
    unlock_time: u64,
    expires_at: Option<u64>,
    occurrences: u32,
    interval: u64,
) -> Result<u64, Error> {
    guard::ensure_initialized(env)?;
    guard::ensure_not_paused(env)?;
    guard::ensure_positive(amount)?;
    sender.require_auth();

    if occurrences == 0 {
        return Err(Error::InvalidInput);
    }
    if occurrences > 1 && interval == 0 {
        return Err(Error::InvalidInput);
    }
    if let Some(expiry) = expires_at {
        if expiry <= unlock_time {
            return Err(Error::InvalidTime);
        }
    }

    let total_locked = amount
        .checked_mul(occurrences as i128)
        .ok_or(Error::Overflow)?;

    let contract = env.current_contract_address();
    let token_client = TokenClient::new(env, &token);
    token_client.transfer(&sender, &contract, &total_locked);

    let now = env.ledger().timestamp();
    let id = storage::next_gift_id(env);
    let status = if now >= unlock_time {
        GiftStatus::Available
    } else {
        GiftStatus::Scheduled
    };

    let gift = Gift {
        id,
        sender: sender.clone(),
        recipient: recipient.clone(),
        token: token.clone(),
        amount,
        total_locked,
        total_claimed: 0,
        message,
        unlock_time,
        interval,
        occurrences,
        claimed_count: 0,
        expires_at,
        status,
        created_at: now,
        claimed_at: None,
    };
    storage::set_gift(env, &gift);
    storage::append_sender_gift(env, &sender, id);
    storage::append_recipient_gift(env, &recipient, id);

    GiftCreated {
        gift_id: id,
        sender,
        recipient,
        token,
        amount,
        total_locked,
        unlock_time,
        occurrences,
    }
    .publish(env);

    Ok(id)
}

pub fn claim_gift(env: &Env, recipient: Address, gift_id: u64) -> Result<i128, Error> {
    guard::ensure_initialized(env)?;
    recipient.require_auth();

    let mut gift = storage::get_gift(env, gift_id).ok_or(Error::NotFound)?;
    if gift.recipient != recipient {
        return Err(Error::Unauthorized);
    }
    if matches!(
        gift.status,
        GiftStatus::Cancelled | GiftStatus::Claimed | GiftStatus::Expired
    ) {
        return Err(Error::InvalidState);
    }

    let now = env.ledger().timestamp();
    if let Some(expiry) = gift.expires_at {
        if now > expiry {
            return Err(Error::GiftExpired);
        }
    }
    if now < gift.unlock_time {
        return Err(Error::NotYetUnlocked);
    }

    let contract = env.current_contract_address();
    let token = TokenClient::new(env, &gift.token);
    token.transfer(&contract, &recipient, &gift.amount);

    gift.claimed_count += 1;
    gift.total_claimed = gift
        .total_claimed
        .checked_add(gift.amount)
        .ok_or(Error::Overflow)?;
    gift.claimed_at = Some(now);

    let fully_claimed = gift.claimed_count >= gift.occurrences;
    if fully_claimed {
        gift.status = GiftStatus::Claimed;
    } else {
        gift.unlock_time = gift
            .unlock_time
            .checked_add(gift.interval)
            .ok_or(Error::Overflow)?;
        gift.status = GiftStatus::Available;
    }
    storage::set_gift(env, &gift);

    GiftClaimed {
        gift_id,
        recipient,
        amount: gift.amount,
        claimed_count: gift.claimed_count,
        next_unlock: gift.unlock_time,
    }
    .publish(env);

    Ok(gift.amount)
}

pub fn cancel_gift(env: &Env, sender: Address, gift_id: u64) -> Result<i128, Error> {
    guard::ensure_initialized(env)?;
    sender.require_auth();

    let mut gift = storage::get_gift(env, gift_id).ok_or(Error::NotFound)?;
    if gift.sender != sender {
        return Err(Error::Unauthorized);
    }
    if matches!(gift.status, GiftStatus::Cancelled | GiftStatus::Claimed) {
        return Err(Error::InvalidState);
    }

    let refund = gift
        .total_locked
        .checked_sub(gift.total_claimed)
        .ok_or(Error::Overflow)?;

    if refund > 0 {
        let contract = env.current_contract_address();
        let token = TokenClient::new(env, &gift.token);
        token.transfer(&contract, &sender, &refund);
    }

    gift.status = GiftStatus::Cancelled;
    storage::set_gift(env, &gift);

    GiftCancelled {
        gift_id,
        sender,
        refunded: refund,
    }
    .publish(env);

    Ok(refund)
}

pub fn get_gift(env: &Env, gift_id: u64) -> Result<Gift, Error> {
    guard::ensure_initialized(env)?;
    let mut gift = storage::get_gift(env, gift_id).ok_or(Error::NotFound)?;

    // Expiry is a time-derived condition; surface it on reads without mutating
    // storage so that state-changing errors remain atomic.
    if matches!(gift.status, GiftStatus::Scheduled | GiftStatus::Available) {
        if let Some(expiry) = gift.expires_at {
            if env.ledger().timestamp() > expiry {
                gift.status = GiftStatus::Expired;
            }
        }
    }
    Ok(gift)
}
