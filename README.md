# Stellix — Smart Contracts

> Soroban smart contracts powering **Stellix**, the AI-powered financial workspace for
> the Stellar ecosystem. This repository implements the on-chain layer that the
> [Stellix frontend](https://github.com/Lumora-Finance/Stellix_front-end) service
> adapters are designed to call.

`Stellix Core` is a single Soroban contract that provides the programmable financial
primitives behind the Stellix workspace:

| Module | What it does | Frontend service |
| --- | --- | --- |
| **Invoice settlement** | Merchants issue invoices; payers settle them on-chain. | `invoiceService` |
| **Recurring payments / automations** | Scheduled, allowance-drawn transfers that anyone can execute when due. | `recurringPaymentService`, `automationService` |
| **Salary splits** | Distribute a payment across destinations by basis points (must total 100%). | `salaryService` |
| **Programmable gifts** | Time-locked, optionally recurring, escrowed gifts that can be claimed or refunded. | `giftService` |

---

## Deployed on Stellar Testnet

| | |
| --- | --- |
| **Contract ID** | [`CDGJVTXDFB6N7IA2ZU7NTOPQHHQY7SVU2F4BSDUAMGWAGMYHFL7PDQYV`](https://stellar.expert/explorer/testnet/contract/CDGJVTXDFB6N7IA2ZU7NTOPQHHQY7SVU2F4BSDUAMGWAGMYHFL7PDQYV) |
| **Network** | Stellar Testnet |
| **Admin** | `GCMCYDMWL7JJOG4XLURAPYXGAAJOYDTT6EX6SGVL6GR6RIEQS76H37WI` |
| **WASM hash** | `c3ebf195ad77a100d8b109db9e794c6d07b3dc3f1d6e3d35bc75bfcdea6c5b0c` |
| **SDK** | `soroban-sdk` v27 (Rust, `no_std`) |
| **Interface version** | `1` |
| **Deploy tx** | [`b7c50a49…`](https://stellar.expert/explorer/testnet/tx/b7c50a49b8886981e1920342d0cc31c99fe5d4a54a30540554ea8a544753a645) |
| **Init tx** | [`02b666f1…`](https://stellar.expert/explorer/testnet/tx/02b666f16c1ae33f54c2df2b6428846b4272776a19fbdd0cec8dc1be9a6481a9) |

A live invoice was created and settled on this deployment (native XLM), producing an
`InvoiceCreated` and `InvoicePaid` event with a real token transfer:

- Create: [`a99abdcb…`](https://stellar.expert/explorer/testnet/tx/a99abdcbe903c1d83744017b342e03d597d07ef65932806695b120ac2ff86608)
- Pay: [`152a66ab…`](https://stellar.expert/explorer/testnet/tx/152a66abbb045b15ad730857e9dd1883533e77e6dff1047dc5f688ee3bd92c69)

---

## Repository layout

```
.
├── Cargo.toml                     # workspace (soroban-sdk = "27", release profile)
├── contracts/stellix-core/
│   ├── Cargo.toml
│   ├── Makefile
│   └── src/
│       ├── lib.rs                 # #[contract] StellixCore + public interface
│       ├── types.rs               # Invoice, RecurringPayment, SalarySplit, Gift, enums
│       ├── error.rs               # #[contracterror] Error
│       ├── storage.rs             # DataKey, typed storage + TTL helpers
│       ├── events.rs              # #[contractevent] definitions
│       ├── guard.rs               # init / pause / amount guards
│       ├── invoice.rs             # invoice settlement
│       ├── recurring.rs           # recurring payments / automations
│       ├── salary.rs              # salary splits
│       ├── gift.rs                # programmable gifts
│       └── test.rs                # 13 unit tests
└── scripts/deploy.sh              # build → deploy → initialize
```

---

## Quick start

Requirements: [Rust](https://rustup.rs/) with the `wasm32v1-none` target and the
[Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli).

```bash
# Install the wasm target (once)
rustup target add wasm32v1-none

# Build the optimized contract
make build            # or: stellar contract build

# Run the test suite
make test             # or: cargo test -p stellix-core

# Lint
make lint             # or: cargo clippy --all-targets -- -D warnings
```

The build output is `target/wasm32v1-none/release/stellix_core.wasm` (~32 KB).

### Deploy

```bash
# Create and fund an identity on testnet (once)
stellar keys generate stellix-deployer
stellar keys fund stellix-deployer --network testnet

# Build, deploy, and initialize in one step
./scripts/deploy.sh
```

Or manually:

```bash
stellar contract deploy \
  --wasm target/wasm32v1-none/release/stellix_core.wasm \
  --source stellix-deployer --network testnet

stellar contract invoke \
  --id <CONTRACT_ID> --source stellix-deployer --network testnet -- \
  initialize --admin <ADMIN_ADDRESS>
```

---

## Interface

### Admin & config

| Function | Description |
| --- | --- |
| `initialize(admin)` | One-time setup; stores the admin. |
| `set_admin(admin, new_admin)` | Rotate the admin (current admin only). |
| `get_admin() -> Address` | Current admin. |
| `set_paused(admin, paused)` | Emergency stop for *new* activity. Claims and cancellations always work. |
| `is_paused() -> bool` | Pause state. |
| `version() -> u32` | Interface version. |

### Invoices

| Function | Description |
| --- | --- |
| `create_invoice(merchant, token, amount, due_date, memo) -> u64` | Merchant issues an invoice. |
| `pay_invoice(payer, invoice_id)` | Payer settles it via SEP-41 allowance (`transfer_from`). |
| `cancel_invoice(merchant, invoice_id)` | Cancel a pending invoice. |
| `get_invoice(invoice_id) -> Invoice` | Read an invoice. |
| `is_invoice_paid(invoice_id) -> bool` | Settlement check. |
| `invoice_count() -> u64` / `get_merchant_invoices(merchant) -> Vec<u64>` | Discovery. |

### Recurring payments (automations)

| Function | Description |
| --- | --- |
| `create_recurring(owner, recipient, token, amount, interval, start_at, end_at, max_occurrences, memo) -> u64` | Create a schedule drawn from `owner`'s allowance. |
| `execute_recurring(executor, schedule_id)` | Execute once when due (any executor; funds move to the fixed recipient). |
| `pause_recurring` / `resume_recurring` / `cancel_recurring(owner, id)` | Manage the schedule. |
| `get_recurring(id) -> RecurringPayment` | Read state (`paid_count`, `next_due`, `status`). |
| `recurring_count()` / `get_owner_recurrings(owner)` | Discovery. |

### Salary splits

| Function | Description |
| --- | --- |
| `create_salary_split(employer, token, rules) -> u64` | Rules are basis points and **must sum to 10,000**. |
| `distribute_salary(employer, split_id, amount)` | Draws `amount` from the employer and pays each destination. |
| `set_salary_split_active(employer, split_id, active)` | Toggle a configuration. |
| `get_salary_split(id) -> SalarySplit` | Read config + totals. |
| `salary_split_count()` / `get_employer_splits(employer)` | Discovery. |

### Programmable gifts

| Function | Description |
| --- | --- |
| `create_gift(sender, recipient, token, amount, message, unlock_time, expires_at, occurrences, interval) -> u64` | Escrows `amount * occurrences` in the contract. |
| `claim_gift(recipient, gift_id) -> i128` | Releases one installment once unlocked. |
| `cancel_gift(sender, gift_id) -> i128` | Refunds the unclaimed balance. |
| `get_gift(id) -> Gift` | Read state; expiry is derived on read. |
| `gift_count()` / `get_sender_gifts` / `get_recipient_gifts` | Discovery. |

---

## Design notes

- **Non-custodial settlement (invoices, recurring, salary).** These flows never hold
  funds. The payer/owner grants the contract an allowance and the contract draws via
  SEP-41 `transfer_from`. Revoking the allowance stops future recurring executions.
- **Escrowed gifts.** Gift funds are transferred into the contract at creation and
  released on claim, or refunded on cancellation. There is no admin withdrawal path,
  so escrowed funds can only move to the sender or recipient.
- **Atomic errors.** Soroban rolls back storage on a returned error, so derived states
  (e.g. gift expiry) are computed on reads rather than persisted mid-failure.
- **Allowance dust.** Salary splits assign the integer-division remainder to the last
  destination, guaranteeing the full amount is distributed.
- **Pause semantics.** `set_paused` blocks new invoices, recurring executions,
  distributions and gift creation, but *always* allows `claim_gift` and `cancel_gift`
  so user funds are never trapped.
- **TTL management.** Instance and persistent entries are periodically extended by the
  contract on writes.

---

## Frontend integration

The frontend already routes all data access through `src/services/*`. Each service
maps to a set of contract calls. Example using `@stellar/stellar-sdk`:

```ts
import * as StellarSdk from "@stellar/stellar-sdk";

const CONTRACT_ID = "CDGJVTXDFB6N7IA2ZU7NTOPQHHQY7SVU2F4BSDUAMGWAGMYHFL7PDQYV";
const RPC_URL = "https://soroban-testnet.stellar.org";
const NETWORK_PASSPHRASE = StellarSdk.Networks.TESTNET;

const server = new StellarSdk.rpc.Server(RPC_URL);
const contract = new StellarSdk.Contract(CONTRACT_ID);

// Read an invoice
async function getInvoice(id: bigint) {
  const op = contract.call("get_invoice", StellarSdk.nativeToScVal(id, { type: "u64" }));
  const tx = new StellarSdk.TransactionBuilder(await server.getAccount(source), {
    fee: "100",
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(op)
    .setTimeout(30)
    .build();
  const sim = await server.simulateTransaction(tx);
  return StellarSdk.scValToNative(sim.result.retval);
}

// Pay an invoice (payer must first approve the contract as spender on the token)
```

Service-to-call mapping:

- `walletService.sendTransaction` → build a settlement (e.g. `pay_invoice` or a direct
  token transfer with `memo`).
- `invoiceService.createInvoice` → `create_invoice`; `markPaid` → `pay_invoice`.
- `recurringPaymentService` / `automationService` → `create_recurring` + executor calls
  `execute_recurring`; `pause`/`resume`/`cancel` map directly.
- `salaryService.updateDistributionRules` → `create_salary_split`;
  `toggleAutoDistribute` → `set_salary_split_active`.
- `giftService.create` → `create_gift`; `claim` → `claim_gift`; `cancel` → `cancel_gift`.

---

## Testing

The suite in `contracts/stellix-core/src/test.rs` covers initialization, admin
rotation, pausing, full invoice lifecycle, recurring execution/pause/resume/completion,
salary split correctness and validation, one-time and recurring gifts, gift expiry,
and cancellation refunds.

```bash
cargo test -p stellix-core
# test result: ok. 13 passed; 0 failed
```

---

## License

MIT
