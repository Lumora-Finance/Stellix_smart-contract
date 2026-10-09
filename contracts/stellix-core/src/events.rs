use soroban_sdk::{contractevent, Address};

#[contractevent(topics = ["Stellix", "initialized"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Initialized {
    pub admin: Address,
}

#[contractevent(topics = ["Stellix", "admin_changed"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminChanged {
    pub admin: Address,
    pub new_admin: Address,
}

#[contractevent(topics = ["Stellix", "paused"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PausedSet {
    pub admin: Address,
    pub paused: bool,
}

#[contractevent(topics = ["Stellix", "invoice_created"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvoiceCreated {
    pub invoice_id: u64,
    pub merchant: Address,
    pub token: Address,
    pub amount: i128,
    pub due_date: u64,
}

#[contractevent(topics = ["Stellix", "invoice_paid"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvoicePaid {
    pub invoice_id: u64,
    pub merchant: Address,
    pub payer: Address,
    pub token: Address,
    pub amount: i128,
}

#[contractevent(topics = ["Stellix", "invoice_cancelled"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvoiceCancelled {
    pub invoice_id: u64,
    pub merchant: Address,
}

#[contractevent(topics = ["Stellix", "recurring_created"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecurringCreated {
    pub schedule_id: u64,
    pub owner: Address,
    pub recipient: Address,
    pub token: Address,
    pub amount: i128,
    pub interval: u64,
    pub next_due: u64,
}

#[contractevent(topics = ["Stellix", "recurring_executed"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecurringExecuted {
    pub schedule_id: u64,
    pub owner: Address,
    pub recipient: Address,
    pub amount: i128,
    pub paid_count: u32,
    pub next_due: u64,
}

#[contractevent(topics = ["Stellix", "recurring_status"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecurringStatusChanged {
    pub schedule_id: u64,
    pub owner: Address,
    pub status: u32,
}

#[contractevent(topics = ["Stellix", "salary_created"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SalarySplitCreated {
    pub split_id: u64,
    pub employer: Address,
    pub token: Address,
    pub rule_count: u32,
}

#[contractevent(topics = ["Stellix", "salary_distributed"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SalaryDistributed {
    pub split_id: u64,
    pub employer: Address,
    pub token: Address,
    pub amount: i128,
    pub distribution_count: u32,
}

#[contractevent(topics = ["Stellix", "salary_status"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SalaryStatusChanged {
    pub split_id: u64,
    pub employer: Address,
    pub active: bool,
}

#[contractevent(topics = ["Stellix", "gift_created"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GiftCreated {
    pub gift_id: u64,
    pub sender: Address,
    pub recipient: Address,
    pub token: Address,
    pub amount: i128,
    pub total_locked: i128,
    pub unlock_time: u64,
    pub occurrences: u32,
}

#[contractevent(topics = ["Stellix", "gift_claimed"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GiftClaimed {
    pub gift_id: u64,
    pub recipient: Address,
    pub amount: i128,
    pub claimed_count: u32,
    pub next_unlock: u64,
}

#[contractevent(topics = ["Stellix", "gift_cancelled"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GiftCancelled {
    pub gift_id: u64,
    pub sender: Address,
    pub refunded: i128,
}
