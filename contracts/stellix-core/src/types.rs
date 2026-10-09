use soroban_sdk::{contracttype, Address, String, Vec};

/// Lifecycle of an on-chain invoice.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvoiceStatus {
    /// Awaiting settlement.
    Pending,
    /// Fully settled.
    Paid,
    /// Cancelled by the merchant before settlement.
    Cancelled,
}

/// An invoice issued by a merchant and settled by a payer.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Invoice {
    pub id: u64,
    pub merchant: Address,
    pub token: Address,
    pub amount: i128,
    pub due_date: u64,
    pub status: InvoiceStatus,
    pub memo: String,
    pub created_at: u64,
    pub paid_at: Option<u64>,
    pub payer: Option<Address>,
}

/// Lifecycle of a recurring payment schedule.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleStatus {
    Active,
    Paused,
    Completed,
    Cancelled,
}

/// A recurring / automated transfer drawn from the owner's allowance.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecurringPayment {
    pub id: u64,
    pub owner: Address,
    pub recipient: Address,
    pub token: Address,
    pub amount: i128,
    /// Seconds between executions.
    pub interval: u64,
    pub start_at: u64,
    pub next_due: u64,
    pub end_at: Option<u64>,
    pub max_occurrences: Option<u32>,
    pub paid_count: u32,
    pub total_paid: i128,
    pub status: ScheduleStatus,
    pub memo: String,
    pub created_at: u64,
}

/// A single destination in a salary split, expressed in basis points (1/100th of a percent).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SplitRule {
    pub label: String,
    pub destination: Address,
    /// Share in basis points. All rules must sum to exactly `10000`.
    pub bps: u32,
}

/// A reusable salary distribution configuration.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SalarySplit {
    pub id: u64,
    pub employer: Address,
    pub token: Address,
    pub rules: Vec<SplitRule>,
    pub distributed_total: i128,
    pub distribution_count: u32,
    pub active: bool,
    pub created_at: u64,
}

/// Lifecycle of a programmable gift.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GiftStatus {
    /// Locked until the first unlock time.
    Scheduled,
    /// Claimable now.
    Available,
    /// Fully claimed.
    Claimed,
    /// Past its expiration with funds still locked.
    Expired,
    /// Cancelled by the sender; remaining funds refunded.
    Cancelled,
}

/// A time-locked, optionally recurring gift. Funds are escrowed in this contract.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Gift {
    pub id: u64,
    pub sender: Address,
    pub recipient: Address,
    pub token: Address,
    /// Amount released per claim.
    pub amount: i128,
    /// `amount * occurrences`, escrowed at creation.
    pub total_locked: i128,
    pub total_claimed: i128,
    pub message: String,
    /// Timestamp of the next claimable installment.
    pub unlock_time: u64,
    /// Seconds between installments (0 for one-time gifts).
    pub interval: u64,
    pub occurrences: u32,
    pub claimed_count: u32,
    pub expires_at: Option<u64>,
    pub status: GiftStatus,
    pub created_at: u64,
    pub claimed_at: Option<u64>,
}
