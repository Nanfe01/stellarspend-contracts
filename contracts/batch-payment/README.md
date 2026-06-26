# `batch-payment`

> Transfer tokens from one sender to multiple recipients in a single transaction.

## Overview

The `batch-payment` contract enables StellarSpend users to execute multi-recipient token transfers in a single contract call. Instead of submitting separate transactions for each payment, the caller provides a list of `(recipient, amount)` pairs and the contract processes them sequentially, emitting per-payment and batch-completion events with a unique reference ID for end-to-end tracking and reconciliation.

## Features

- **Multi-Recipient Transfers**: Pay any number of recipients in one call
- **Reference ID Tracking**: Auto-generated unique batch reference ID for reconciliation
- **Per-Payment Events**: Each individual transfer emits a structured event
- **Batch Completion Event**: Summary event with total count and amount
- **Token Agnostic**: Works with any Soroban token contract (USDC, XLM, etc.)
- **Amount Validation**: Rejects zero or negative payment amounts

---

## Public API

### `batch_transfer`

```rust
pub fn batch_transfer(
    env: Env,
    from: Address,
    token: Address,
    payments: Vec<Payment>,
) -> String
```

| Parameter  | Type           | Description                                           |
| ---------- | -------------- | ----------------------------------------------------- |
| `from`     | `Address`      | Sender address (must authorize the call)              |
| `token`    | `Address`      | Token contract address (e.g., USDC, native XLM)      |
| `payments` | `Vec<Payment>` | List of recipient/amount pairs                        |

**Behaviour:**

1. Requires `from.require_auth()`.
2. Generates a unique `batch_reference_id` via `generate_transaction_reference_id`.
3. Iterates through `payments`:
   - Validates `amount > 0` (panics otherwise).
   - Calls `token.transfer(from, recipient, amount)`.
   - Emits a per-payment event.
4. Emits a batch-completion event with total count and amount.

**Returns:** `String` — the unique batch reference ID for tracking.

---

## Storage Layout

| DataKey Variant        | Storage Type | Value Type | Description                           |
| ---------------------- | ------------ | ---------- | ------------------------------------- |
| `batch_ref_counter`    | Instance     | `u64`      | Auto-incrementing reference counter   |

> The counter is managed by the `shared::utils::generate_transaction_reference_id` helper and stored as a generic `Symbol` key.

---

## Events

| Event Name        | Topics                                          | Payload                    |
| ----------------- | ----------------------------------------------- | -------------------------- |
| `payment`         | `("payment", batch_reference_id, recipient)`    | `(token, amount)`          |
| `batch_complete`  | `("batch", "complete", batch_reference_id)`     | `(count, total_amount)`    |

---

## Types

### `Payment`

```rust
pub struct Payment {
    pub recipient: Address,
    pub amount: i128,
}
```

---

## Usage Example

```rust
use soroban_sdk::{Env, Address, Vec};
use batch_payment::{BatchPaymentContract, Payment};

let payments = vec![
    &env,
    Payment { recipient: alice.clone(), amount: 1_000_000 },
    Payment { recipient: bob.clone(), amount: 2_500_000 },
    Payment { recipient: carol.clone(), amount: 750_000 },
];

// Execute batch payment
let ref_id = contract.batch_transfer(&env, &sender, &usdc_token, &payments);

// ref_id can be used to look up all related events
```

---

## Testing

```bash
cargo test -p batch-payment
```

---

## Design Notes

- **No batch size limit enforced**: Unlike other batch contracts, `batch-payment` does not enforce a `MAX_BATCH_SIZE`. The effective limit is determined by Soroban's per-transaction CPU and memory budgets.
- **Reference ID generation**: The contract delegates reference ID creation to the `shared` crate's `generate_transaction_reference_id`, which combines the sender address with an auto-incrementing counter to produce a deterministic, unique string per batch.
- **Fail-fast on invalid amounts**: A single invalid payment amount panics the entire transaction. This is intentional — partial payment batches could create reconciliation issues. If partial-failure semantics are needed, use the `escrow` contract's batch operations instead.

---

## License

MIT
