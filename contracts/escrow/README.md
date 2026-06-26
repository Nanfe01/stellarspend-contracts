# `escrow`

> Lock, release, and batch-reverse escrowed funds on Stellar with full audit trails.

## Overview

The `escrow` contract provides a complete escrow lifecycle for StellarSpend: depositors lock funds for a recipient, and an admin (or depositor/arbiter) can release or reverse the escrow. The contract supports **batch operations** for efficiently processing multiple reversals or releases in a single transaction, with partial failure handling so that one bad escrow doesn't block the rest.

## Features

- **Full Escrow Lifecycle**: Create → Release or Reverse
- **Batch Reversals**: Reverse multiple escrows in one call with per-item success/failure
- **Batch Releases**: Release multiple escrows to recipients in one call
- **Single Escrow Operations**: Release individual escrows via admin, depositor, or arbiter
- **Arbiter Support**: Optional third-party arbiter can authorize releases
- **Aggregate Statistics**: Tracks total batches, escrows processed, and amounts moved
- **Comprehensive Events**: Every operation emits structured events for indexers
- **Partial Failure Tolerance**: Batch operations continue past individual failures

---

## Public API

### Initialization

```rust
pub fn initialize(env: Env, admin: Address, token: Address)
```

| Parameter | Type      | Description                                |
| --------- | --------- | ------------------------------------------ |
| `admin`   | `Address` | Contract administrator                     |
| `token`   | `Address` | Token contract address used for all escrows|

Sets up the contract with an admin and token. Initializes all counters to zero. Panics with `AlreadyInitialized` if called twice.

---

### Write Functions

#### `create_escrow`

```rust
pub fn create_escrow(
    env: Env,
    depositor: Address,
    recipient: Address,
    arbiter: Option<Address>,
    amount: i128,
    deadline: u64,
) -> u64
```

| Parameter   | Type              | Description                                 |
| ----------- | ----------------- | ------------------------------------------- |
| `depositor` | `Address`         | Sender who locks funds (must authorize)     |
| `recipient` | `Address`         | Intended receiver of funds                  |
| `arbiter`   | `Option<Address>` | Optional third-party who can release        |
| `amount`    | `i128`            | Amount to escrow (must be > 0)              |
| `deadline`  | `u64`             | Deadline ledger sequence                    |

**Returns:** `u64` — the new escrow ID.

Transfers `amount` from depositor to the contract, creates an `Escrow` record, and emits `escrow_created`.

#### `batch_reverse_escrows`

```rust
pub fn batch_reverse_escrows(
    env: Env,
    caller: Address,
    requests: Vec<ReversalRequest>,
) -> BatchReversalResult
```

Admin-only. Validates and reverses up to `MAX_BATCH_SIZE` (100) escrows. Uses a two-pass approach: validate all, then execute. Returns detailed per-escrow results.

#### `batch_release_escrows`

```rust
pub fn batch_release_escrows(
    env: Env,
    caller: Address,
    requests: Vec<ReleaseRequest>,
) -> BatchReleaseResult
```

Releases multiple escrows to their recipients. Admin or depositor can call. Same two-pass validate-then-execute pattern.

#### `release_escrow`

```rust
pub fn release_escrow(env: Env, caller: Address, escrow_id: u64)
```

Releases a single escrow. Callable by admin, depositor, or arbiter.

#### `set_admin`

```rust
pub fn set_admin(env: Env, current_admin: Address, new_admin: Address)
```

Transfers admin privileges. Current admin must authorize.

---

### Read / Query Functions

| Function                         | Returns    | Description                        |
| -------------------------------- | ---------- | ---------------------------------- |
| `get_escrow(escrow_id)`          | `Option<Escrow>` | Lookup escrow by ID          |
| `get_user_escrows(user)`         | `Vec<u64>` | All escrow IDs for a depositor     |
| `get_admin()`                    | `Address`  | Current admin address              |
| `get_escrow_counter()`           | `u64`      | Total escrows ever created         |
| `get_total_reversal_batches()`   | `u64`      | Count of reversal batches          |
| `get_total_escrows_reversed()`   | `u64`      | Count of reversed escrows          |
| `get_total_amount_reversed()`    | `i128`     | Sum of all reversed amounts        |
| `get_total_release_batches()`    | `u64`      | Count of release batches           |
| `get_total_escrows_released()`   | `u64`      | Count of released escrows          |
| `get_total_amount_released()`    | `i128`     | Sum of all released amounts        |

---

## Storage Layout

| DataKey Variant              | Storage Type | Value Type   | Description                          |
| ---------------------------- | ------------ | ------------ | ------------------------------------ |
| `Admin`                      | Instance     | `Address`    | Contract admin                       |
| `Token`                      | Instance     | `Address`    | Token contract for all escrows       |
| `EscrowCounter`              | Instance     | `u64`        | Auto-incrementing escrow ID          |
| `TotalReversalBatches`       | Instance     | `u64`        | Lifetime reversal batch count        |
| `TotalEscrowsReversed`       | Instance     | `u64`        | Lifetime reversed escrow count       |
| `TotalAmountReversed`        | Instance     | `i128`       | Lifetime reversed amount             |
| `TotalReleaseBatches`        | Instance     | `u64`        | Lifetime release batch count         |
| `TotalEscrowsReleased`       | Instance     | `u64`        | Lifetime released escrow count       |
| `TotalAmountReleased`        | Instance     | `i128`       | Lifetime released amount             |
| `Escrow(u64)`                | Persistent   | `Escrow`     | Individual escrow record by ID       |
| `UserEscrows(Address)`       | Persistent   | `Vec<u64>`   | Escrow ID list per depositor         |

---

## Events

| Event Name             | Topics                             | Payload                                                  |
| ---------------------- | ---------------------------------- | -------------------------------------------------------- |
| `escrow_created`       | `("escrow", "created")`            | `(escrow_id, depositor, recipient, arbiter, amount)`     |
| `batch_reversal_started`| `("escrow", "rev_start")`         | `(batch_id, request_count)`                              |
| `reversal_success`     | `("escrow", "rev_ok", batch_id)`   | `(escrow_id, depositor, amount)`                         |
| `reversal_failure`     | `("escrow", "rev_fail", batch_id)` | `(escrow_id, error_code)`                                |
| `batch_reversal_completed`| `("escrow", "rev_done", batch_id)`| `(successful, failed, total_reversed)`                  |
| `escrow_released`      | `("escrow", "released")`           | `(escrow_id, recipient, amount)`                         |
| `batch_release_started`| `("escrow", "rel_start")`          | `(batch_id, request_count)`                              |
| `release_success`      | `("escrow", "rel_ok", batch_id)`   | `(escrow_id, recipient, amount)`                         |
| `release_failure`      | `("escrow", "rel_fail", batch_id)` | `(escrow_id, error_code)`                                |
| `batch_release_completed`| `("escrow", "rel_done", batch_id)`| `(successful, failed, total_released)`                  |

---

## Error Codes

| Code | Name                 | Description                         |
| ---- | -------------------- | ----------------------------------- |
| 1    | `NotInitialized`     | Contract has not been initialized   |
| 2    | `Unauthorized`       | Caller is not admin/depositor/arbiter|
| 3    | `EmptyBatch`         | Batch request vector is empty       |
| 4    | `BatchTooLarge`      | Batch exceeds `MAX_BATCH_SIZE` (100)|
| 5    | `InvalidAmount`      | Amount is zero or negative          |
| 6    | `EscrowNotFound`     | No escrow exists with given ID      |
| 7    | `AlreadyInitialized` | Contract was already initialized    |

---

## Types

### `EscrowStatus`

```rust
pub enum EscrowStatus {
    Active,    // Funds locked
    Released,  // Funds sent to recipient
    Reversed,  // Funds returned to depositor
}
```

### `Escrow`

```rust
pub struct Escrow {
    pub escrow_id: u64,
    pub depositor: Address,
    pub recipient: Address,
    pub arbiter: Option<Address>,
    pub token: Address,
    pub amount: i128,
    pub status: EscrowStatus,
    pub created_at: u64,
    pub deadline: u64,
}
```

### Batch Result Types

- `BatchReversalResult` — aggregated reversal outcome with per-escrow `ReversalResult` entries
- `BatchReleaseResult` — aggregated release outcome with per-escrow `ReleaseResult` entries

---

## Usage Example

```rust
use soroban_sdk::{Env, Address, Vec};
use escrow::{EscrowContract, ReversalRequest};

// 1. Initialize
contract.initialize(&env, &admin, &token);

// 2. Create an escrow
let id = contract.create_escrow(&env, &depositor, &recipient, &None, 1_000_000, deadline);

// 3. Release it
contract.release_escrow(&env, &admin, id);

// 4. Or batch-reverse multiple escrows
let result = contract.batch_reverse_escrows(
    &env,
    &admin,
    &vec![&env, ReversalRequest { escrow_id: 1 }, ReversalRequest { escrow_id: 2 }],
);
assert_eq!(result.successful, 2);
```

---

## Testing

```bash
cargo test -p escrow
```

---

## Design Notes

- **Two-pass batch processing**: All requests are validated before any state changes, preventing partial state corruption if validation would otherwise fail mid-batch.
- **`MAX_BATCH_SIZE = 100`**: Chosen to stay within Soroban CPU/memory budgets while still providing meaningful batching benefits.
- **Overflow protection**: `checked_add` is used throughout for amount aggregation, saturating to `i128::MAX` rather than panicking.
- **Arbiter pattern**: The optional arbiter provides a three-party escrow model where a neutral third party can authorize release without requiring the depositor.

---

## License

MIT
