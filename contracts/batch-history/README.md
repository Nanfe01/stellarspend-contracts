# `batch-history`

> Retrieve transaction history for multiple users in a single contract call.

## Overview

The `batch-history` contract enables efficient batch retrieval of user transaction histories on Stellar. Instead of making individual cross-contract calls for each user, callers can request histories for up to 100 users in a single invocation, reducing round-trips and overall transaction costs.

## Features

- **Batch Retrieval**: Fetch transaction histories for up to 100 users in one call
- **Structured Events**: Emits per-user and aggregate batch events for indexer integration
- **Caller Authorization**: Requires `require_auth` from the requesting address
- **Size Guard**: Rejects batches exceeding `MAX_BATCH_SIZE` (100) to prevent resource exhaustion

---

## Public API

### `retrieve_histories`

```rust
pub fn retrieve_histories(
    env: Env,
    requester: Address,
    users: Vec<Address>,
) -> Vec<UserHistory>
```

| Parameter   | Type            | Description                                           |
| ----------- | --------------- | ----------------------------------------------------- |
| `requester` | `Address`       | The caller requesting histories (must authorize)      |
| `users`     | `Vec<Address>`  | List of user addresses to retrieve histories for      |

**Behaviour:**

1. Validates `requester` authorization via `require_auth()`.
2. Returns empty `Vec` if `users` is empty.
3. Panics if `users.len() > MAX_BATCH_SIZE` (100).
4. Iterates through each user, constructs a `UserHistory`, and emits a per-user `UserHistoryRetrievedEvent`.
5. Emits an aggregate `BatchHistoryRetrievedEvent` with request/response counts.

**Returns:** `Vec<UserHistory>` — one entry per requested user.

---

## Storage Layout

This contract currently uses **no persistent storage**. History records are constructed on-the-fly (with a placeholder for future cache/storage lookup).

| DataKey Variant | Storage Type | Value Type    | Description                         |
| --------------- | ------------ | ------------- | ----------------------------------- |
| _(none yet)_    | Temporary    | `UserHistory` | Planned: per-user history cache     |

---

## Events

| Event Name       | Topics                                    | Payload                               |
| ---------------- | ----------------------------------------- | ------------------------------------- |
| `user_retrieved` | `("history", "user")`                     | `UserHistoryRetrievedEvent { user }`  |
| `batch_done`     | `("history", "batch")`                    | `BatchHistoryRetrievedEvent { requested_users, returned_records }` |

---

## Types

### `TransactionRecord`

```rust
pub struct TransactionRecord {
    pub amount: i128,
    pub timestamp: u64,
    pub description: String,
}
```

### `UserHistory`

```rust
pub struct UserHistory {
    pub user: Address,
    pub transactions: Vec<TransactionRecord>,
}
```

### Event Structs

```rust
pub struct BatchHistoryRetrievedEvent {
    pub requested_users: u32,
    pub returned_records: u32,
}

pub struct UserHistoryRetrievedEvent {
    pub user: Address,
}
```

---

## Usage Example

```rust
use soroban_sdk::{Env, Address, Vec};
use batch_history::BatchHistoryContract;

// Call from an authorized requester
let histories = contract.retrieve_histories(
    &env,
    &requester,
    &vec![&env, user_a.clone(), user_b.clone()],
);

assert_eq!(histories.len(), 2);
```

---

## Testing

```bash
cargo test -p batch-history
```

---

## Design Notes

- **`MAX_BATCH_SIZE = 100`** — chosen to fit comfortably within Soroban CPU/memory budgets for a single invocation.
- The logic module contains a **commented-out cache lookup** path using `env.storage().temporary()`, intended for future optimisation where frequently-accessed histories are cached in temporary storage.
- The contract is intentionally stateless today; it serves as a batch-aggregation layer over future history storage contracts.

---

## License

MIT
