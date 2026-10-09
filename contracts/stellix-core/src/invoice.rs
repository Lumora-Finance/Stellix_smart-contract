use crate::{
    error::Error,
    events::{InvoiceCancelled, InvoiceCreated, InvoicePaid},
    guard, storage,
    types::{Invoice, InvoiceStatus},
};
use soroban_sdk::{token::TokenClient, Address, Env, String};

pub fn create_invoice(
    env: &Env,
    merchant: Address,
    token: Address,
    amount: i128,
    due_date: u64,
    memo: String,
) -> Result<u64, Error> {
    guard::ensure_initialized(env)?;
    guard::ensure_not_paused(env)?;
    guard::ensure_positive(amount)?;
    merchant.require_auth();

    storage::extend_instance(env);

    let id = storage::next_invoice_id(env);
    let invoice = Invoice {
        id,
        merchant: merchant.clone(),
        token: token.clone(),
        amount,
        due_date,
        status: InvoiceStatus::Pending,
        memo,
        created_at: env.ledger().timestamp(),
        paid_at: None,
        payer: None,
    };
    storage::set_invoice(env, &invoice);
    storage::append_merchant_invoice(env, &merchant, id);

    InvoiceCreated {
        invoice_id: id,
        merchant,
        token,
        amount,
        due_date,
    }
    .publish(env);

    Ok(id)
}

pub fn pay_invoice(env: &Env, payer: Address, invoice_id: u64) -> Result<(), Error> {
    guard::ensure_initialized(env)?;
    guard::ensure_not_paused(env)?;
    payer.require_auth();

    let mut invoice = storage::get_invoice(env, invoice_id).ok_or(Error::NotFound)?;
    if invoice.status != InvoiceStatus::Pending {
        return Err(Error::InvalidState);
    }

    let token = TokenClient::new(env, &invoice.token);
    token.transfer_from(
        &env.current_contract_address(),
        &payer,
        &invoice.merchant,
        &invoice.amount,
    );

    invoice.status = InvoiceStatus::Paid;
    invoice.paid_at = Some(env.ledger().timestamp());
    invoice.payer = Some(payer.clone());
    storage::set_invoice(env, &invoice);

    InvoicePaid {
        invoice_id,
        merchant: invoice.merchant,
        payer,
        token: invoice.token,
        amount: invoice.amount,
    }
    .publish(env);

    Ok(())
}

pub fn cancel_invoice(env: &Env, merchant: Address, invoice_id: u64) -> Result<(), Error> {
    guard::ensure_initialized(env)?;
    guard::ensure_not_paused(env)?;
    merchant.require_auth();

    let mut invoice = storage::get_invoice(env, invoice_id).ok_or(Error::NotFound)?;
    if invoice.merchant != merchant {
        return Err(Error::Unauthorized);
    }
    if invoice.status != InvoiceStatus::Pending {
        return Err(Error::InvalidState);
    }

    invoice.status = InvoiceStatus::Cancelled;
    storage::set_invoice(env, &invoice);

    InvoiceCancelled {
        invoice_id,
        merchant,
    }
    .publish(env);

    Ok(())
}

pub fn get_invoice(env: &Env, invoice_id: u64) -> Result<Invoice, Error> {
    guard::ensure_initialized(env)?;
    storage::get_invoice(env, invoice_id).ok_or(Error::NotFound)
}

pub fn is_invoice_paid(env: &Env, invoice_id: u64) -> bool {
    matches!(
        storage::get_invoice(env, invoice_id),
        Some(Invoice {
            status: InvoiceStatus::Paid,
            ..
        })
    )
}
