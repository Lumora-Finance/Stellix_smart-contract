#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token::{StellarAssetClient, TokenClient},
    Address, Env, String,
};

const START: u64 = 1_000_000;
const WEEK: u64 = 7 * 86_400;

fn setup(env: &Env) -> (Address, Address, Address) {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let contract_id = env.register(StellixCore, ());
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    (admin, contract_id, sac.address())
}

fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
}

fn approve(env: &Env, token: &Address, from: &Address, spender: &Address, amount: i128) {
    let live_until = env.ledger().sequence() + 100_000;
    TokenClient::new(env, token).approve(from, spender, &amount, &live_until);
}

fn balance(env: &Env, token: &Address, of: &Address) -> i128 {
    TokenClient::new(env, token).balance(of)
}

fn rule(env: &Env, label: &str, dest: &Address, bps: u32) -> SplitRule {
    SplitRule {
        label: String::from_str(env, label),
        destination: dest.clone(),
        bps,
    }
}

#[test]
fn initialize_once() {
    let env = Env::default();
    env.ledger().set_timestamp(START);
    let (admin, contract_id, _token) = setup(&env);
    let client = StellixCoreClient::new(&env, &contract_id);

    client.initialize(&admin);
    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.version(), 1);
    assert!(!client.is_paused());

    assert!(client.try_initialize(&admin).is_err());
}

#[test]
fn set_admin_and_pause() {
    let env = Env::default();
    env.ledger().set_timestamp(START);
    let (admin, contract_id, _token) = setup(&env);
    let client = StellixCoreClient::new(&env, &contract_id);
    client.initialize(&admin);

    let new_admin = Address::generate(&env);
    client.set_admin(&admin, &new_admin);
    assert_eq!(client.get_admin(), new_admin);

    client.set_paused(&new_admin, &true);
    assert!(client.is_paused());
    client.set_paused(&new_admin, &false);
    assert!(!client.is_paused());
}

#[test]
fn invoice_lifecycle() {
    let env = Env::default();
    env.ledger().set_timestamp(START);
    let (admin, contract_id, token) = setup(&env);
    let client = StellixCoreClient::new(&env, &contract_id);
    client.initialize(&admin);

    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    mint(&env, &token, &payer, 10_000);
    approve(&env, &token, &payer, &contract_id, 10_000);

    let memo = String::from_str(&env, "INV-104");
    let id = client.create_invoice(&merchant, &token, &2_450, &(START + WEEK), &memo);
    assert_eq!(id, 1);
    assert_eq!(client.invoice_count(), 1);
    assert!(!client.is_invoice_paid(&id));

    let invoice = client.get_invoice(&id);
    assert_eq!(invoice.amount, 2_450);
    assert_eq!(invoice.status, InvoiceStatus::Pending);
    assert_eq!(client.get_merchant_invoices(&merchant), soroban_sdk::vec![&env, id]);

    client.pay_invoice(&payer, &id);
    assert!(client.is_invoice_paid(&id));
    assert_eq!(balance(&env, &token, &merchant), 2_450);
    assert_eq!(balance(&env, &token, &payer), 7_550);

    let paid = client.get_invoice(&id);
    assert_eq!(paid.status, InvoiceStatus::Paid);
    assert_eq!(paid.payer, Some(payer.clone()));
    assert!(paid.paid_at.is_some());

    // Paying twice must fail.
    assert!(client.try_pay_invoice(&payer, &id).is_err());
}

#[test]
fn invoice_cancel_then_pay_fails() {
    let env = Env::default();
    env.ledger().set_timestamp(START);
    let (admin, contract_id, token) = setup(&env);
    let client = StellixCoreClient::new(&env, &contract_id);
    client.initialize(&admin);

    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    mint(&env, &token, &payer, 100);
    approve(&env, &token, &payer, &contract_id, 100);

    let id = client.create_invoice(&merchant, &token, &100, &(START + WEEK), &String::from_str(&env, "x"));
    client.cancel_invoice(&merchant, &id);
    assert_eq!(client.get_invoice(&id).status, InvoiceStatus::Cancelled);
    assert!(client.try_pay_invoice(&payer, &id).is_err());

    // A different merchant cannot cancel.
    let intruder = Address::generate(&env);
    let id2 = client.create_invoice(&merchant, &token, &50, &(START + WEEK), &String::from_str(&env, "y"));
    assert!(client.try_cancel_invoice(&intruder, &id2).is_err());
}

#[test]
fn recurring_payment_flow() {
    let env = Env::default();
    env.ledger().set_timestamp(START);
    let (admin, contract_id, token) = setup(&env);
    let client = StellixCoreClient::new(&env, &contract_id);
    client.initialize(&admin);

    let owner = Address::generate(&env);
    let recipient = Address::generate(&env);
    mint(&env, &token, &owner, 1_000);
    approve(&env, &token, &owner, &contract_id, 1_000);

    let id = client.create_recurring(
        &owner,
        &recipient,
        &token,
        &50,
        &WEEK,
        &START,
        &None,
        &Some(3u32),
        &String::from_str(&env, "family"),
    );
    assert_eq!(id, 1);
    assert_eq!(client.recurring_count(), 1);
    assert_eq!(client.get_owner_recurrings(&owner), soroban_sdk::vec![&env, id]);

    // Not due until the start timestamp (== now, so due).
    client.execute_recurring(&owner, &id);
    assert_eq!(balance(&env, &token, &recipient), 50);

    // Too early for the next installment.
    assert!(client.try_execute_recurring(&owner, &id).is_err());

    env.ledger().set_timestamp(START + WEEK);
    client.execute_recurring(&owner, &id);
    assert_eq!(balance(&env, &token, &recipient), 100);

    // Pause blocks execution.
    client.pause_recurring(&owner, &id);
    env.ledger().set_timestamp(START + 2 * WEEK);
    assert!(client.try_execute_recurring(&owner, &id).is_err());
    client.resume_recurring(&owner, &id);
    client.execute_recurring(&owner, &id);
    assert_eq!(balance(&env, &token, &recipient), 150);

    // Third occurrence completes the schedule.
    let done = client.get_recurring(&id);
    assert_eq!(done.paid_count, 3);
    assert_eq!(done.status, ScheduleStatus::Completed);
    assert!(client.try_execute_recurring(&owner, &id).is_err());
}

#[test]
fn recurring_cancel_and_resume_errors() {
    let env = Env::default();
    env.ledger().set_timestamp(START);
    let (admin, contract_id, token) = setup(&env);
    let client = StellixCoreClient::new(&env, &contract_id);
    client.initialize(&admin);

    let owner = Address::generate(&env);
    let recipient = Address::generate(&env);

    let id = client.create_recurring(
        &owner,
        &recipient,
        &token,
        &10,
        &WEEK,
        &START,
        &None,
        &None,
        &String::from_str(&env, "x"),
    );
    // Cannot resume an active schedule.
    assert!(client.try_resume_recurring(&owner, &id).is_err());
    client.cancel_recurring(&owner, &id);
    assert_eq!(client.get_recurring(&id).status, ScheduleStatus::Cancelled);
    assert!(client.try_cancel_recurring(&owner, &id).is_err());
}

#[test]
fn salary_split_distribution() {
    let env = Env::default();
    env.ledger().set_timestamp(START);
    let (admin, contract_id, token) = setup(&env);
    let client = StellixCoreClient::new(&env, &contract_id);
    client.initialize(&admin);

    let employer = Address::generate(&env);
    let personal = Address::generate(&env);
    let savings = Address::generate(&env);
    let rent = Address::generate(&env);
    let family = Address::generate(&env);
    mint(&env, &token, &employer, 10_000);
    approve(&env, &token, &employer, &contract_id, 10_000);

    let rules = soroban_sdk::vec![
        &env,
        rule(&env, "Personal", &personal, 6_000),
        rule(&env, "Savings", &savings, 2_000),
        rule(&env, "Rent", &rent, 1_000),
        rule(&env, "Family", &family, 1_000),
    ];
    let id = client.create_salary_split(&employer, &token, &rules);
    assert_eq!(client.salary_split_count(), 1);

    client.distribute_salary(&employer, &id, &2_000);
    assert_eq!(balance(&env, &token, &personal), 1_200);
    assert_eq!(balance(&env, &token, &savings), 400);
    assert_eq!(balance(&env, &token, &rent), 200);
    assert_eq!(balance(&env, &token, &family), 200);
    assert_eq!(balance(&env, &token, &employer), 8_000);

    let split = client.get_salary_split(&id);
    assert_eq!(split.distributed_total, 2_000);
    assert_eq!(split.distribution_count, 1);

    client.set_salary_split_active(&employer, &id, &false);
    assert!(client.try_distribute_salary(&employer, &id, &100).is_err());
}

#[test]
fn salary_split_rejects_invalid_total() {
    let env = Env::default();
    env.ledger().set_timestamp(START);
    let (admin, contract_id, token) = setup(&env);
    let client = StellixCoreClient::new(&env, &contract_id);
    client.initialize(&admin);

    let employer = Address::generate(&env);
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let bad = soroban_sdk::vec![
        &env,
        rule(&env, "A", &a, 5_000),
        rule(&env, "B", &b, 4_000),
    ];
    assert!(client.try_create_salary_split(&employer, &token, &bad).is_err());
}

#[test]
fn gift_one_time_unlock() {
    let env = Env::default();
    env.ledger().set_timestamp(START);
    let (admin, contract_id, token) = setup(&env);
    let client = StellixCoreClient::new(&env, &contract_id);
    client.initialize(&admin);

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    mint(&env, &token, &sender, 1_000);

    let unlock = START + WEEK;
    let id = client.create_gift(
        &sender,
        &recipient,
        &token,
        &100,
        &String::from_str(&env, "Happy Birthday"),
        &unlock,
        &None,
        &1u32,
        &0u64,
    );
    // Funds are escrowed immediately.
    assert_eq!(balance(&env, &token, &contract_id), 100);
    assert_eq!(balance(&env, &token, &sender), 900);
    let gift = client.get_gift(&id);
    assert_eq!(gift.status, GiftStatus::Scheduled);
    assert_eq!(gift.total_locked, 100);

    // Cannot claim before unlock.
    assert!(client.try_claim_gift(&recipient, &id).is_err());

    env.ledger().set_timestamp(unlock);
    assert_eq!(client.claim_gift(&recipient, &id), 100);
    assert_eq!(balance(&env, &token, &recipient), 100);
    assert_eq!(balance(&env, &token, &contract_id), 0);
    let claimed = client.get_gift(&id);
    assert_eq!(claimed.status, GiftStatus::Claimed);
    assert_eq!(claimed.claimed_count, 1);

    assert!(client.try_claim_gift(&recipient, &id).is_err());
    assert_eq!(client.get_recipient_gifts(&recipient), soroban_sdk::vec![&env, id]);
}

#[test]
fn gift_recurring_installments() {
    let env = Env::default();
    env.ledger().set_timestamp(START);
    let (admin, contract_id, token) = setup(&env);
    let client = StellixCoreClient::new(&env, &contract_id);
    client.initialize(&admin);

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    mint(&env, &token, &sender, 1_000);

    let id = client.create_gift(
        &sender,
        &recipient,
        &token,
        &50,
        &String::from_str(&env, "Weekly gift"),
        &START,
        &None,
        &3u32,
        &WEEK,
    );
    assert_eq!(balance(&env, &token, &contract_id), 150);

    assert_eq!(client.claim_gift(&recipient, &id), 50);
    assert_eq!(client.get_gift(&id).status, GiftStatus::Available);

    // Next installment is locked.
    assert!(client.try_claim_gift(&recipient, &id).is_err());

    env.ledger().set_timestamp(START + WEEK);
    client.claim_gift(&recipient, &id);
    env.ledger().set_timestamp(START + 2 * WEEK);
    client.claim_gift(&recipient, &id);

    assert_eq!(balance(&env, &token, &recipient), 150);
    assert_eq!(balance(&env, &token, &contract_id), 0);
    assert_eq!(client.get_gift(&id).status, GiftStatus::Claimed);
}

#[test]
fn gift_cancel_refunds_remaining() {
    let env = Env::default();
    env.ledger().set_timestamp(START);
    let (admin, contract_id, token) = setup(&env);
    let client = StellixCoreClient::new(&env, &contract_id);
    client.initialize(&admin);

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    mint(&env, &token, &sender, 1_000);

    let id = client.create_gift(
        &sender,
        &recipient,
        &token,
        &50,
        &String::from_str(&env, "gift"),
        &START,
        &None,
        &3u32,
        &WEEK,
    );
    client.claim_gift(&recipient, &id);

    let refund = client.cancel_gift(&sender, &id);
    assert_eq!(refund, 100);
    assert_eq!(balance(&env, &token, &sender), 950);
    assert_eq!(balance(&env, &token, &recipient), 50);
    assert_eq!(client.get_gift(&id).status, GiftStatus::Cancelled);
}

#[test]
fn gift_expiry_marks_expired() {
    let env = Env::default();
    env.ledger().set_timestamp(START);
    let (admin, contract_id, token) = setup(&env);
    let client = StellixCoreClient::new(&env, &contract_id);
    client.initialize(&admin);

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    mint(&env, &token, &sender, 100);

    let unlock = START + WEEK;
    let expiry = unlock + WEEK;
    let id = client.create_gift(
        &sender,
        &recipient,
        &token,
        &100,
        &String::from_str(&env, "expires"),
        &unlock,
        &Some(expiry),
        &1u32,
        &0u64,
    );

    env.ledger().set_timestamp(expiry + 1);
    assert!(client.try_claim_gift(&recipient, &id).is_err());
    assert_eq!(client.get_gift(&id).status, GiftStatus::Expired);

    // Sender can still reclaim after expiry.
    assert_eq!(client.cancel_gift(&sender, &id), 100);
    assert_eq!(balance(&env, &token, &sender), 100);
}

#[test]
fn pause_blocks_new_activity_but_not_claims() {
    let env = Env::default();
    env.ledger().set_timestamp(START);
    let (admin, contract_id, token) = setup(&env);
    let client = StellixCoreClient::new(&env, &contract_id);
    client.initialize(&admin);

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let merchant = Address::generate(&env);
    mint(&env, &token, &sender, 500);

    let id = client.create_gift(
        &sender,
        &recipient,
        &token,
        &100,
        &String::from_str(&env, "g"),
        &START,
        &None,
        &1u32,
        &0u64,
    );

    client.set_paused(&admin, &true);
    assert!(client.try_create_invoice(&merchant, &token, &1, &START, &String::from_str(&env, "x")).is_err());
    assert!(client
        .try_create_recurring(&sender, &recipient, &token, &1, &WEEK, &START, &None, &None, &String::from_str(&env, "r"))
        .is_err());

    // Claims still work while paused.
    assert_eq!(client.claim_gift(&recipient, &id), 100);
}
