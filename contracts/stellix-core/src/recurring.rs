use crate::{
    error::Error,
    events::{RecurringCreated, RecurringExecuted, RecurringStatusChanged},
    guard, storage,
    types::{RecurringPayment, ScheduleStatus},
};
use soroban_sdk::{token::TokenClient, Address, Env, String};

#[allow(clippy::too_many_arguments)]
pub fn create_recurring(
    env: &Env,
    owner: Address,
    recipient: Address,
    token: Address,
    amount: i128,
    interval: u64,
    start_at: u64,
    end_at: Option<u64>,
    max_occurrences: Option<u32>,
    memo: String,
) -> Result<u64, Error> {
    guard::ensure_initialized(env)?;
    guard::ensure_not_paused(env)?;
    guard::ensure_positive(amount)?;
    owner.require_auth();

    if interval == 0 {
        return Err(Error::InvalidInput);
    }
    if max_occurrences == Some(0) {
        return Err(Error::InvalidInput);
    }
    let now = env.ledger().timestamp();
    let start = if start_at == 0 { now } else { start_at };
    if let Some(end) = end_at {
        if end <= start {
            return Err(Error::InvalidTime);
        }
    }

    storage::extend_instance(env);

    let id = storage::next_recurring_id(env);
    let schedule = RecurringPayment {
        id,
        owner: owner.clone(),
        recipient: recipient.clone(),
        token: token.clone(),
        amount,
        interval,
        start_at: start,
        next_due: start,
        end_at,
        max_occurrences,
        paid_count: 0,
        total_paid: 0,
        status: ScheduleStatus::Active,
        memo,
        created_at: now,
    };
    storage::set_recurring(env, &schedule);
    storage::append_owner_recurring(env, &owner, id);

    RecurringCreated {
        schedule_id: id,
        owner,
        recipient,
        token,
        amount,
        interval,
        next_due: start,
    }
    .publish(env);

    Ok(id)
}

pub fn execute_recurring(env: &Env, executor: Address, schedule_id: u64) -> Result<(), Error> {
    guard::ensure_initialized(env)?;
    guard::ensure_not_paused(env)?;
    executor.require_auth();

    let mut schedule =
        storage::get_recurring(env, schedule_id).ok_or(Error::NotFound)?;
    if schedule.status != ScheduleStatus::Active {
        return Err(Error::InvalidState);
    }

    let now = env.ledger().timestamp();

    if let Some(max) = schedule.max_occurrences {
        if schedule.paid_count >= max {
            schedule.status = ScheduleStatus::Completed;
            storage::set_recurring(env, &schedule);
            return Err(Error::NoOccurrencesRemaining);
        }
    }
    if let Some(end) = schedule.end_at {
        if now > end {
            schedule.status = ScheduleStatus::Completed;
            storage::set_recurring(env, &schedule);
            return Err(Error::InvalidState);
        }
    }
    if now < schedule.next_due {
        return Err(Error::NotDue);
    }

    let token = TokenClient::new(env, &schedule.token);
    token.transfer_from(
        &env.current_contract_address(),
        &schedule.owner,
        &schedule.recipient,
        &schedule.amount,
    );

    schedule.paid_count += 1;
    schedule.total_paid = schedule
        .total_paid
        .checked_add(schedule.amount)
        .ok_or(Error::Overflow)?;
    schedule.next_due = schedule
        .next_due
        .checked_add(schedule.interval)
        .ok_or(Error::Overflow)?;

    let exhausted = schedule
        .max_occurrences
        .map(|max| schedule.paid_count >= max)
        .unwrap_or(false);
    let past_end = schedule
        .end_at
        .map(|end| schedule.next_due > end)
        .unwrap_or(false);
    if exhausted || past_end {
        schedule.status = ScheduleStatus::Completed;
    }

    storage::set_recurring(env, &schedule);

    RecurringExecuted {
        schedule_id,
        owner: schedule.owner,
        recipient: schedule.recipient,
        amount: schedule.amount,
        paid_count: schedule.paid_count,
        next_due: schedule.next_due,
    }
    .publish(env);

    Ok(())
}

fn set_status(
    env: &Env,
    owner: Address,
    schedule_id: u64,
    from: ScheduleStatus,
    to: ScheduleStatus,
) -> Result<(), Error> {
    guard::ensure_initialized(env)?;
    owner.require_auth();

    let mut schedule =
        storage::get_recurring(env, schedule_id).ok_or(Error::NotFound)?;
    if schedule.owner != owner {
        return Err(Error::Unauthorized);
    }
    if schedule.status != from {
        return Err(Error::InvalidState);
    }

    schedule.status = to;
    storage::set_recurring(env, &schedule);

    RecurringStatusChanged {
        schedule_id,
        owner,
        status: status_code(to),
    }
    .publish(env);

    Ok(())
}

pub fn pause_recurring(env: &Env, owner: Address, schedule_id: u64) -> Result<(), Error> {
    set_status(
        env,
        owner,
        schedule_id,
        ScheduleStatus::Active,
        ScheduleStatus::Paused,
    )
}

pub fn resume_recurring(env: &Env, owner: Address, schedule_id: u64) -> Result<(), Error> {
    guard::ensure_initialized(env)?;
    owner.require_auth();

    let mut schedule =
        storage::get_recurring(env, schedule_id).ok_or(Error::NotFound)?;
    if schedule.owner != owner {
        return Err(Error::Unauthorized);
    }
    if schedule.status != ScheduleStatus::Paused {
        return Err(Error::InvalidState);
    }
    if let Some(max) = schedule.max_occurrences {
        if schedule.paid_count >= max {
            return Err(Error::NoOccurrencesRemaining);
        }
    }

    schedule.status = ScheduleStatus::Active;
    storage::set_recurring(env, &schedule);

    RecurringStatusChanged {
        schedule_id,
        owner,
        status: status_code(ScheduleStatus::Active),
    }
    .publish(env);

    Ok(())
}

pub fn cancel_recurring(env: &Env, owner: Address, schedule_id: u64) -> Result<(), Error> {
    guard::ensure_initialized(env)?;
    owner.require_auth();

    let mut schedule =
        storage::get_recurring(env, schedule_id).ok_or(Error::NotFound)?;
    if schedule.owner != owner {
        return Err(Error::Unauthorized);
    }
    if matches!(
        schedule.status,
        ScheduleStatus::Cancelled | ScheduleStatus::Completed
    ) {
        return Err(Error::InvalidState);
    }

    schedule.status = ScheduleStatus::Cancelled;
    storage::set_recurring(env, &schedule);

    RecurringStatusChanged {
        schedule_id,
        owner,
        status: status_code(ScheduleStatus::Cancelled),
    }
    .publish(env);

    Ok(())
}

pub fn get_recurring(env: &Env, schedule_id: u64) -> Result<RecurringPayment, Error> {
    guard::ensure_initialized(env)?;
    storage::get_recurring(env, schedule_id).ok_or(Error::NotFound)
}

fn status_code(status: ScheduleStatus) -> u32 {
    match status {
        ScheduleStatus::Active => 0,
        ScheduleStatus::Paused => 1,
        ScheduleStatus::Completed => 2,
        ScheduleStatus::Cancelled => 3,
    }
}
