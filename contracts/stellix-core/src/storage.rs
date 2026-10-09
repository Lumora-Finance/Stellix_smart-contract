use crate::types::{Gift, Invoice, RecurringPayment, SalarySplit};
use soroban_sdk::{contracttype, Address, Env, Vec};

/// Maximum number of split rules allowed in a salary configuration.
pub const MAX_SPLIT_RULES: u32 = 20;
/// Total basis points used for percentage math.
pub const BPS_DENOMINATOR: u32 = 10_000;

const DAY_IN_LEDGERS: u32 = 17_280;
const TTL_THRESHOLD: u32 = DAY_IN_LEDGERS;
const TTL_EXTEND_TO: u32 = DAY_IN_LEDGERS * 30;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Paused,
    InvoiceCount,
    Invoice(u64),
    MerchantInvoices(Address),
    RecurringCount,
    Recurring(u64),
    OwnerRecurrings(Address),
    SalaryCount,
    SalarySplit(u64),
    EmployerSplits(Address),
    GiftCount,
    Gift(u64),
    SenderGifts(Address),
    RecipientGifts(Address),
}

// ── Instance-level helpers ────────────────────────────────────────────────────

pub fn extend_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
}

pub fn bump(env: &Env, key: &DataKey) {
    env.storage()
        .persistent()
        .extend_ttl(key, TTL_THRESHOLD, TTL_EXTEND_TO);
}

pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn is_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false)
}

pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&DataKey::Paused, &paused);
}

fn next_id(env: &Env, key: &DataKey) -> u64 {
    let current: u64 = env.storage().instance().get(key).unwrap_or(0);
    let next = current + 1;
    env.storage().instance().set(key, &next);
    next
}

pub fn next_invoice_id(env: &Env) -> u64 {
    next_id(env, &DataKey::InvoiceCount)
}

pub fn next_recurring_id(env: &Env) -> u64 {
    next_id(env, &DataKey::RecurringCount)
}

pub fn next_salary_id(env: &Env) -> u64 {
    next_id(env, &DataKey::SalaryCount)
}

pub fn next_gift_id(env: &Env) -> u64 {
    next_id(env, &DataKey::GiftCount)
}

fn counter(env: &Env, key: &DataKey) -> u64 {
    env.storage().instance().get(key).unwrap_or(0)
}

pub fn invoice_count(env: &Env) -> u64 {
    counter(env, &DataKey::InvoiceCount)
}

pub fn recurring_count(env: &Env) -> u64 {
    counter(env, &DataKey::RecurringCount)
}

pub fn salary_count(env: &Env) -> u64 {
    counter(env, &DataKey::SalaryCount)
}

pub fn gift_count(env: &Env) -> u64 {
    counter(env, &DataKey::GiftCount)
}

// ── Invoices ──────────────────────────────────────────────────────────────────

pub fn set_invoice(env: &Env, invoice: &Invoice) {
    let key = DataKey::Invoice(invoice.id);
    env.storage().persistent().set(&key, invoice);
    bump(env, &key);
}

pub fn get_invoice(env: &Env, id: u64) -> Option<Invoice> {
    env.storage().persistent().get(&DataKey::Invoice(id))
}

pub fn append_merchant_invoice(env: &Env, merchant: &Address, id: u64) {
    let key = DataKey::MerchantInvoices(merchant.clone());
    let mut ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));
    ids.push_back(id);
    env.storage().persistent().set(&key, &ids);
    bump(env, &key);
}

pub fn get_merchant_invoices(env: &Env, merchant: &Address) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::MerchantInvoices(merchant.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

// ── Recurring payments ────────────────────────────────────────────────────────

pub fn set_recurring(env: &Env, recurring: &RecurringPayment) {
    let key = DataKey::Recurring(recurring.id);
    env.storage().persistent().set(&key, recurring);
    bump(env, &key);
}

pub fn get_recurring(env: &Env, id: u64) -> Option<RecurringPayment> {
    env.storage().persistent().get(&DataKey::Recurring(id))
}

pub fn append_owner_recurring(env: &Env, owner: &Address, id: u64) {
    let key = DataKey::OwnerRecurrings(owner.clone());
    let mut ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));
    ids.push_back(id);
    env.storage().persistent().set(&key, &ids);
    bump(env, &key);
}

pub fn get_owner_recurrings(env: &Env, owner: &Address) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::OwnerRecurrings(owner.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

// ── Salary splits ─────────────────────────────────────────────────────────────

pub fn set_salary_split(env: &Env, split: &SalarySplit) {
    let key = DataKey::SalarySplit(split.id);
    env.storage().persistent().set(&key, split);
    bump(env, &key);
}

pub fn get_salary_split(env: &Env, id: u64) -> Option<SalarySplit> {
    env.storage().persistent().get(&DataKey::SalarySplit(id))
}

pub fn append_employer_split(env: &Env, employer: &Address, id: u64) {
    let key = DataKey::EmployerSplits(employer.clone());
    let mut ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));
    ids.push_back(id);
    env.storage().persistent().set(&key, &ids);
    bump(env, &key);
}

pub fn get_employer_splits(env: &Env, employer: &Address) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::EmployerSplits(employer.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

// ── Gifts ─────────────────────────────────────────────────────────────────────

pub fn set_gift(env: &Env, gift: &Gift) {
    let key = DataKey::Gift(gift.id);
    env.storage().persistent().set(&key, gift);
    bump(env, &key);
}

pub fn get_gift(env: &Env, id: u64) -> Option<Gift> {
    env.storage().persistent().get(&DataKey::Gift(id))
}

fn append_to_list(env: &Env, key: DataKey, id: u64) {
    let mut ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));
    ids.push_back(id);
    env.storage().persistent().set(&key, &ids);
    bump(env, &key);
}

pub fn append_sender_gift(env: &Env, sender: &Address, id: u64) {
    append_to_list(env, DataKey::SenderGifts(sender.clone()), id);
}

pub fn append_recipient_gift(env: &Env, recipient: &Address, id: u64) {
    append_to_list(env, DataKey::RecipientGifts(recipient.clone()), id);
}

pub fn get_sender_gifts(env: &Env, sender: &Address) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::SenderGifts(sender.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn get_recipient_gifts(env: &Env, recipient: &Address) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::RecipientGifts(recipient.clone()))
        .unwrap_or_else(|| Vec::new(env))
}
