use crate::{
    error::Error,
    events::{SalaryDistributed, SalarySplitCreated, SalaryStatusChanged},
    guard, storage,
    types::{SalarySplit, SplitRule},
};
use soroban_sdk::{token::TokenClient, Address, Env, Vec};

pub fn create_salary_split(
    env: &Env,
    employer: Address,
    token: Address,
    rules: Vec<SplitRule>,
) -> Result<u64, Error> {
    guard::ensure_initialized(env)?;
    guard::ensure_not_paused(env)?;
    employer.require_auth();

    let count = rules.len();
    if count == 0 {
        return Err(Error::InvalidSplit);
    }
    if count > storage::MAX_SPLIT_RULES {
        return Err(Error::TooManyRules);
    }

    let mut total: u32 = 0;
    for rule in rules.iter() {
        if rule.bps == 0 {
            return Err(Error::InvalidSplit);
        }
        total = total.checked_add(rule.bps).ok_or(Error::Overflow)?;
    }
    if total != storage::BPS_DENOMINATOR {
        return Err(Error::InvalidSplit);
    }

    storage::extend_instance(env);

    let id = storage::next_salary_id(env);
    let split = SalarySplit {
        id,
        employer: employer.clone(),
        token: token.clone(),
        rules: rules.clone(),
        distributed_total: 0,
        distribution_count: 0,
        active: true,
        created_at: env.ledger().timestamp(),
    };
    storage::set_salary_split(env, &split);
    storage::append_employer_split(env, &employer, id);

    SalarySplitCreated {
        split_id: id,
        employer,
        token,
        rule_count: count,
    }
    .publish(env);

    Ok(id)
}

pub fn distribute_salary(
    env: &Env,
    employer: Address,
    split_id: u64,
    amount: i128,
) -> Result<(), Error> {
    guard::ensure_initialized(env)?;
    guard::ensure_not_paused(env)?;
    guard::ensure_positive(amount)?;
    employer.require_auth();

    let mut split =
        storage::get_salary_split(env, split_id).ok_or(Error::NotFound)?;
    if split.employer != employer {
        return Err(Error::Unauthorized);
    }
    if !split.active {
        return Err(Error::InvalidState);
    }

    let token = TokenClient::new(env, &split.token);
    let contract = env.current_contract_address();
    let len = split.rules.len();
    let mut remaining = amount;

    let mut i = 0u32;
    while i < len {
        let rule = split.rules.get(i).ok_or(Error::NotFound)?;
        let share = if i + 1 == len {
            remaining
        } else {
            let weighted = amount
                .checked_mul(rule.bps as i128)
                .ok_or(Error::Overflow)?;
            let part = weighted / (storage::BPS_DENOMINATOR as i128);
            remaining = remaining.checked_sub(part).ok_or(Error::Overflow)?;
            part
        };

        if share > 0 {
            token.transfer_from(&contract, &employer, &rule.destination, &share);
        }
        i += 1;
    }

    split.distributed_total = split
        .distributed_total
        .checked_add(amount)
        .ok_or(Error::Overflow)?;
    split.distribution_count += 1;
    storage::set_salary_split(env, &split);

    SalaryDistributed {
        split_id,
        employer,
        token: split.token,
        amount,
        distribution_count: split.distribution_count,
    }
    .publish(env);

    Ok(())
}

pub fn set_salary_split_active(
    env: &Env,
    employer: Address,
    split_id: u64,
    active: bool,
) -> Result<(), Error> {
    guard::ensure_initialized(env)?;
    employer.require_auth();

    let mut split =
        storage::get_salary_split(env, split_id).ok_or(Error::NotFound)?;
    if split.employer != employer {
        return Err(Error::Unauthorized);
    }

    split.active = active;
    storage::set_salary_split(env, &split);

    SalaryStatusChanged {
        split_id,
        employer,
        active,
    }
    .publish(env);

    Ok(())
}

pub fn get_salary_split(env: &Env, split_id: u64) -> Result<SalarySplit, Error> {
    guard::ensure_initialized(env)?;
    storage::get_salary_split(env, split_id).ok_or(Error::NotFound)
}
