#![no_std]

mod error;
mod events;
mod gift;
mod guard;
mod invoice;
mod recurring;
mod salary;
mod storage;
mod types;

#[cfg(test)]
mod test;

use error::Error;
use events::{AdminChanged, Initialized, PausedSet};
use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};
use types::{Gift, Invoice, RecurringPayment, SalarySplit, SplitRule};

pub use types::{GiftStatus, InvoiceStatus, ScheduleStatus};

/// Stellix Core: the on-chain settlement layer powering Stellix's programmable
/// financial operations on Stellar.
///
/// It bundles four features that back the Stellix workspace:
/// * **Invoice settlement** — merchants issue invoices, payers settle them on-chain.
/// * **Recurring payments** — automated, allowance-drawn transfers on a schedule.
/// * **Salary splits** — distribute a payment across destinations by basis points.
/// * **Programmable gifts** — time-locked, optionally recurring, escrowed gifts.
#[contract]
pub struct StellixCore;

#[contractimpl]
impl StellixCore {
    // ── Admin ────────────────────────────────────────────────────────────────

    /// Initializes the contract with the given admin. Can only be called once.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if storage::has_admin(&env) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        storage::extend_instance(&env);

        Initialized { admin }.publish(&env);
        Ok(())
    }

    /// Transfers the admin role to `new_admin`. Requires the current admin.
    pub fn set_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), Error> {
        guard::ensure_initialized(&env)?;
        admin.require_auth();
        if storage::get_admin(&env) != Some(admin.clone()) {
            return Err(Error::Unauthorized);
        }
        storage::set_admin(&env, &new_admin);

        AdminChanged { admin, new_admin }.publish(&env);
        Ok(())
    }

    /// Returns the current admin.
    pub fn get_admin(env: Env) -> Result<Address, Error> {
        storage::get_admin(&env).ok_or(Error::NotInitialized)
    }

    /// Pauses or unpauses new financial activity. Claims and cancellations are
    /// always allowed so users' funds are never trapped.
    pub fn set_paused(env: Env, admin: Address, paused: bool) -> Result<(), Error> {
        guard::ensure_initialized(&env)?;
        admin.require_auth();
        if storage::get_admin(&env) != Some(admin.clone()) {
            return Err(Error::Unauthorized);
        }
        storage::set_paused(&env, paused);

        PausedSet { admin, paused }.publish(&env);
        Ok(())
    }

    pub fn is_paused(env: Env) -> bool {
        storage::is_paused(&env)
    }

    /// Contract interface version, bumped on breaking changes.
    pub fn version() -> u32 {
        1
    }

    // ── Invoices ─────────────────────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    pub fn create_invoice(
        env: Env,
        merchant: Address,
        token: Address,
        amount: i128,
        due_date: u64,
        memo: String,
    ) -> Result<u64, Error> {
        invoice::create_invoice(&env, merchant, token, amount, due_date, memo)
    }

    pub fn pay_invoice(env: Env, payer: Address, invoice_id: u64) -> Result<(), Error> {
        invoice::pay_invoice(&env, payer, invoice_id)
    }

    pub fn cancel_invoice(env: Env, merchant: Address, invoice_id: u64) -> Result<(), Error> {
        invoice::cancel_invoice(&env, merchant, invoice_id)
    }

    pub fn get_invoice(env: Env, invoice_id: u64) -> Result<Invoice, Error> {
        invoice::get_invoice(&env, invoice_id)
    }

    pub fn is_invoice_paid(env: Env, invoice_id: u64) -> bool {
        invoice::is_invoice_paid(&env, invoice_id)
    }

    pub fn invoice_count(env: Env) -> u64 {
        storage::invoice_count(&env)
    }

    pub fn get_merchant_invoices(env: Env, merchant: Address) -> Vec<u64> {
        storage::get_merchant_invoices(&env, &merchant)
    }

    // ── Recurring payments ───────────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    pub fn create_recurring(
        env: Env,
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
        recurring::create_recurring(
            &env,
            owner,
            recipient,
            token,
            amount,
            interval,
            start_at,
            end_at,
            max_occurrences,
            memo,
        )
    }

    pub fn execute_recurring(env: Env, executor: Address, schedule_id: u64) -> Result<(), Error> {
        recurring::execute_recurring(&env, executor, schedule_id)
    }

    pub fn pause_recurring(env: Env, owner: Address, schedule_id: u64) -> Result<(), Error> {
        recurring::pause_recurring(&env, owner, schedule_id)
    }

    pub fn resume_recurring(env: Env, owner: Address, schedule_id: u64) -> Result<(), Error> {
        recurring::resume_recurring(&env, owner, schedule_id)
    }

    pub fn cancel_recurring(env: Env, owner: Address, schedule_id: u64) -> Result<(), Error> {
        recurring::cancel_recurring(&env, owner, schedule_id)
    }

    pub fn get_recurring(env: Env, schedule_id: u64) -> Result<RecurringPayment, Error> {
        recurring::get_recurring(&env, schedule_id)
    }

    pub fn recurring_count(env: Env) -> u64 {
        storage::recurring_count(&env)
    }

    pub fn get_owner_recurrings(env: Env, owner: Address) -> Vec<u64> {
        storage::get_owner_recurrings(&env, &owner)
    }

    // ── Salary splits ────────────────────────────────────────────────────────

    pub fn create_salary_split(
        env: Env,
        employer: Address,
        token: Address,
        rules: Vec<SplitRule>,
    ) -> Result<u64, Error> {
        salary::create_salary_split(&env, employer, token, rules)
    }

    pub fn distribute_salary(
        env: Env,
        employer: Address,
        split_id: u64,
        amount: i128,
    ) -> Result<(), Error> {
        salary::distribute_salary(&env, employer, split_id, amount)
    }

    pub fn set_salary_split_active(
        env: Env,
        employer: Address,
        split_id: u64,
        active: bool,
    ) -> Result<(), Error> {
        salary::set_salary_split_active(&env, employer, split_id, active)
    }

    pub fn get_salary_split(env: Env, split_id: u64) -> Result<SalarySplit, Error> {
        salary::get_salary_split(&env, split_id)
    }

    pub fn salary_split_count(env: Env) -> u64 {
        storage::salary_count(&env)
    }

    pub fn get_employer_splits(env: Env, employer: Address) -> Vec<u64> {
        storage::get_employer_splits(&env, &employer)
    }

    // ── Programmable gifts ───────────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    pub fn create_gift(
        env: Env,
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
        gift::create_gift(
            &env,
            sender,
            recipient,
            token,
            amount,
            message,
            unlock_time,
            expires_at,
            occurrences,
            interval,
        )
    }

    pub fn claim_gift(env: Env, recipient: Address, gift_id: u64) -> Result<i128, Error> {
        gift::claim_gift(&env, recipient, gift_id)
    }

    pub fn cancel_gift(env: Env, sender: Address, gift_id: u64) -> Result<i128, Error> {
        gift::cancel_gift(&env, sender, gift_id)
    }

    pub fn get_gift(env: Env, gift_id: u64) -> Result<Gift, Error> {
        gift::get_gift(&env, gift_id)
    }

    pub fn gift_count(env: Env) -> u64 {
        storage::gift_count(&env)
    }

    pub fn get_sender_gifts(env: Env, sender: Address) -> Vec<u64> {
        storage::get_sender_gifts(&env, &sender)
    }

    pub fn get_recipient_gifts(env: Env, recipient: Address) -> Vec<u64> {
        storage::get_recipient_gifts(&env, &recipient)
    }
}
