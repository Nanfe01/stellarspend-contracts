# `fee`

> Fee collection, escrow, tiered pricing, and cycle-based release management for StellarSpend.

## Overview

The `fee` contract manages the complete lifecycle of transaction fees within StellarSpend. Fees are collected into an escrow balance, grouped by billing cycles, and released to a treasury address on demand. The contract supports fee decay (reduced fees for active users), user tier assignment, configurable basis-point rates with min/max bounds, and a lock mechanism to freeze configuration changes.

## Features

- **Basis-Point Fee Calculation**: Configurable fee rate in BPS (default: 500 = 5%)
- **Fee Decay**: Active users pay lower fees based on their last activity timestamp
- **Escrow Model**: Collected fees are held in escrow until explicitly released per cycle
- **Cycle Management**: Fees are grouped into cycles; admins release or roll over per cycle
- **User Tiers**: Assign bronze/silver/gold/platinum tiers for tiered pricing
- **Min/Max Fee Bounds**: Enforce floor and ceiling on individual fee amounts
- **Batch Collection**: Collect fees for multiple amounts in a single call
- **Lock/Unlock**: Admin can lock the contract to prevent configuration changes
- **Reconciliation**: Compare stored vs. calculated balances for integrity checks
- **Consolidated Storage**: Config and stats use single-struct reads for gas efficiency

---

## Public API

### Initialization

```rust
pub fn initialize(env: Env, admin: Address, token: Address, treasury: Address, fee_bps: u32, initial_cycle: u64)
pub fn init(env: Env, admin: Address, token: Address, treasury: Address)
```

`initialize` sets up the contract with custom fee BPS and initial cycle. `init` is a convenience wrapper using defaults (300 BPS, cycle 1).

---

### Fee Collection

#### `collect_fee`

```rust
pub fn collect_fee(env: Env, payer: Address, amount: i128) -> i128
```

Collects a fee from `payer` on `amount`, applying fee decay based on user activity. Returns the updated pending fee balance.

#### `collect_fee_batch`

```rust
pub fn collect_fee_batch(env: Env, payer: Address, amounts: Vec<i128>) -> BatchFeeResult
```

Batch-collects fees for multiple amounts. Max batch size: 100.

#### `preview_batch_fee`

```rust
pub fn preview_batch_fee(env: Env, _payer: Address, amounts: Vec<i128>) -> i128
```

Dry-run fee calculation without state changes.

---

### Fee Release & Rollover

#### `release_fees`

```rust
pub fn release_fees(env: Env, admin: Address, cycle: u64) -> i128
```

Admin-only. Releases all pending fees for a given cycle to the treasury.

#### `rollover_fees`

```rust
pub fn rollover_fees(env: Env, admin: Address, next_cycle: u64) -> i128
```

Admin-only. Rolls pending fees from the current cycle into `next_cycle`.

---

### Configuration

| Function              | Auth  | Description                                   |
| --------------------- | ----- | --------------------------------------------- |
| `set_fee_bps(bps)`    | Admin | Update fee rate in basis points               |
| `set_treasury(addr)`  | Admin | Update treasury address                       |
| `set_min_fee(amount)` | Admin | Set minimum fee floor                         |
| `set_max_fee(amount)` | Admin | Set maximum fee ceiling                       |
| `reset_fee_config()`  | Admin | Reset to defaults (500 BPS, min_fee = 0)      |
| `lock()`              | Admin | Lock contract — prevents config changes       |
| `unlock()`            | Admin | Unlock contract — re-enables config changes   |

All configuration functions require the contract to be unlocked.

---

### User Tiers

```rust
pub fn set_user_tier(env: Env, admin: Address, user: Address, tier: Symbol)
pub fn get_user_tier(env: Env, user: Address) -> Option<Symbol>
pub fn remove_user_tier(env: Env, admin: Address, user: Address)
```

Valid tiers: `bronze`, `silver`, `gold`, `platinum`.

---

### Read / Query Functions

| Function                    | Returns                | Description                         |
| --------------------------- | ---------------------- | ----------------------------------- |
| `get_admin()`               | `Address`              | Current admin                       |
| `get_token()`               | `Address`              | Token contract address              |
| `get_treasury()`            | `Address`              | Treasury address                    |
| `get_fee_bps()`             | `u32`                  | Current fee rate in BPS             |
| `get_min_fee()`             | `i128`                 | Minimum fee floor                   |
| `get_max_fee()`             | `i128`                 | Maximum fee ceiling                 |
| `is_locked()`               | `bool`                 | Whether config is locked            |
| `get_current_cycle()`       | `u64`                  | Active billing cycle                |
| `get_escrow_balance()`      | `i128`                 | Total escrowed fee balance          |
| `get_fee_balance()`         | `i128`                 | Alias for `get_escrow_balance`      |
| `get_pending_fees(cycle)`   | `i128`                 | Pending fees for a specific cycle   |
| `get_total_collected()`     | `i128`                 | Lifetime total collected            |
| `get_total_released()`      | `i128`                 | Lifetime total released             |
| `get_total_batch_calls()`   | `u64`                  | Lifetime batch call count           |
| `get_last_active(user)`     | `u64`                  | Last activity timestamp for a user  |
| `get_reconciliation_status()` | `ReconciliationResult` | Balance integrity check           |
| `calculate_fee_amount(amount, bps)` | `i128`        | Pure fee calculation                |
| `validate_config(bps, min)` | `bool`                 | Validate config parameters          |

---

## Storage Layout

### Instance Storage (Consolidated)

| DataKey Variant | Value Type  | Description                                                      |
| --------------- | ----------- | ---------------------------------------------------------------- |
| `FeeConfig`     | `FeeConfig` | Admin, token, treasury, fee_bps, min/max fee, is_locked, cycle   |
| `FeeStats`      | `FeeStats`  | Escrow balance, total collected, total released, batch call count |

### Persistent Storage

| DataKey Variant          | Value Type | Description                         |
| ------------------------ | ---------- | ----------------------------------- |
| `PendingFees(u64)`       | `i128`     | Pending fee amount per cycle        |
| `UserActivity(Address)`  | `u64`      | Last activity timestamp per user    |
| `UserTier(Address)`      | `Symbol`   | Fee tier per user                   |

> **Storage Optimization (#484):** Config and stats are stored as consolidated structs (`FeeConfig`, `FeeStats`) instead of individual keys, reducing instance-storage reads from 12 to 2.

---

## Events

| Event Name         | Topics                          | Payload                              |
| ------------------ | ------------------------------- | ------------------------------------ |
| `fee_collected`    | `("fee", "collected")`          | `(payer, amount)`                    |
| `fee_escrowed`     | `("fee", "escrowed")`           | `(payer, amount, cycle)`             |
| `fee_batched`      | `("fee", "batched")`            | `(payer, total, batch_size, cycle)`  |
| `fee_released`     | `("fee", "released")`           | `(cycle, amount, treasury)`          |
| `fee_rolled`       | `("fee", "rolled")`             | `(from_cycle, to_cycle, amount)`     |
| `fee_reconciled`   | `("fee", "reconciled")`         | `(stored_balance, calculated)`       |
| `fee_bps_updated`  | `("config", "fee_bps")`         | `(new_bps)`                          |
| `treasury_updated` | `("config", "treasury")`        | `(new_treasury)`                     |
| `min_fee_updated`  | `("config", "min_fee")`         | `(new_min_fee)`                      |
| `fee_reset`        | `("config", "reset")`           | `(admin)`                            |
| `locked`           | `("config", "locked")`          | `()`                                 |
| `unlocked`         | `("config", "unlocked")`        | `()`                                 |
| `tier_set`         | `("tier", "set")`               | `(admin, user, tier)`                |
| `tier_removed`     | `("tier", "removed")`           | `(admin, user)`                      |

---

## Error Codes

| Code | Name                | Description                              |
| ---- | ------------------- | ---------------------------------------- |
| 1    | `NotInitialized`    | Contract has not been initialized        |
| 2    | `Unauthorized`      | Caller is not admin                      |
| 3    | `Locked`            | Config changes blocked while locked      |
| 4    | `InvalidAmount`     | Amount is zero, negative, or below min   |
| 5    | `EmptyBatch`        | Batch is empty                           |
| 6    | `BatchTooLarge`     | Batch exceeds `MAX_BATCH_SIZE` (100)     |
| 7    | `Overflow`          | Arithmetic overflow in fee calculation   |
| 8    | `InsufficientEscrow`| Not enough escrowed to release           |
| 9    | `InvalidCycle`      | Rollover target cycle is not ahead       |
| 10   | `InvalidConfig`     | Invalid configuration parameter          |
| 11   | `NoPendingFees`     | No fees pending for the specified cycle  |
| 12   | `InvalidTier`       | Tier name is not recognized              |

---

## Types

### `FeeConfig`

```rust
pub struct FeeConfig {
    pub admin: Address,
    pub token: Address,
    pub treasury: Address,
    pub fee_bps: u32,
    pub min_fee: i128,
    pub max_fee: i128,
    pub is_locked: bool,
    pub current_cycle: u64,
}
```

### `FeeStats`

```rust
pub struct FeeStats {
    pub escrow_balance: i128,
    pub total_collected: i128,
    pub total_released: i128,
    pub total_batch_calls: u64,
}
```

### `BatchFeeResult`

```rust
pub struct BatchFeeResult {
    pub batch_size: u32,
    pub total_amount: i128,
    pub cycle: u64,
    pub pending_fees: i128,
}
```

### Constants

| Constant          | Value       | Description                  |
| ----------------- | ----------- | ---------------------------- |
| `MAX_BATCH_SIZE`  | `100`       | Maximum items per batch      |
| `MAX_FEE_BPS`    | `10_000`    | Maximum fee rate (100%)      |
| `DEFAULT_FEE_BPS` | `500`       | Default fee rate (5%)        |
| `DEFAULT_MIN_FEE` | `0`         | Default minimum fee          |
| `DEFAULT_MAX_FEE` | `1_000_000` | Default maximum fee          |

---

## Usage Example

```rust
use soroban_sdk::{Env, Address, Vec};
use fee::FeeContract;

// 1. Initialize with defaults
contract.init(&env, &admin, &token, &treasury);

// 2. Collect a fee
let pending = contract.collect_fee(&env, &payer, 10_000_000);

// 3. Batch collect
let result = contract.collect_fee_batch(
    &env,
    &payer,
    &vec![&env, 5_000_000, 3_000_000],
);

// 4. Release fees for cycle 1
let released = contract.release_fees(&env, &admin, 1);

// 5. Check reconciliation
let status = contract.get_reconciliation_status(&env);
```

---

## Testing

```bash
cargo test -p fee
```

---

## Design Notes

- **Storage Optimization (#484)**: The `FeeConfig` and `FeeStats` structs consolidate what were previously 12+ separate instance-storage keys into 2 struct reads. This significantly reduces Soroban I/O overhead and rent costs.
- **Fee Decay**: The `calculate_fee_decay` function reduces fees for users who have been recently active, incentivising platform engagement.
- **Lock/Unlock pattern**: When locked, no configuration can change. This enables "freeze periods" during audits or before cycle releases.
- **Reconciliation**: The `reconcile` function independently computes what the escrow balance should be from collected/released totals and compares it to the stored value, providing a safety net against state drift.

---

## License

MIT
