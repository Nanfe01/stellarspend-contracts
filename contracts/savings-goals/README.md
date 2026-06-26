# `savings-goals`

> Batch savings goal creation, milestone tracking, contributions, withdrawals, and goal cloning on Stellar.

## Overview

The `savings-goals` contract is StellarSpend's savings management system. It allows admins to create savings goals for multiple users in a single batch call, and users to contribute toward their goals, track milestones at 25/50/75/100%, withdraw funds (with optional lock periods and early-withdrawal penalties), clone existing goals, and reverse recent contributions. The contract is designed for high-throughput batch operations with O(n) processing, partial failure support, and minimized storage writes.

## Features

- **Batch Goal Creation**: Create savings goals for up to 100 users in one call
- **Batch Milestones**: Mark milestone achievements for multiple goals at once
- **Contributions**: Users contribute toward goals with automatic milestone emission
- **Contribution Reversal**: Reverse contributions within a 24-hour window
- **Withdrawals**: Withdraw funds with lock period enforcement and early-withdrawal penalties
- **Goal Cloning**: Clone an existing goal with inherited settings and a fresh balance
- **Goal Renaming**: Rename an active goal (unique per user)
- **Progress Tracking**: Query progress percentage, snapshots, and prerequisites
- **Prerequisite Goals**: Goals can depend on other goals being completed first
- **Expiration**: Goals can expire after a configured duration
- **Partial Failure**: Batch operations continue past individual failures

---

## Public API

### Initialization

```rust
pub fn initialize(env: Env, admin: Address)
```

Sets the admin, initializes `LastBatchId`, `LastGoalId`, `TotalGoalsCreated`, and `TotalBatchesProcessed` to zero.

---

### Batch Operations

#### `batch_set_savings_goals`

```rust
pub fn batch_set_savings_goals(
    env: Env,
    caller: Address,                    // Admin only
    requests: Vec<SavingsGoalRequest>,
) -> BatchGoalResult
```

Creates goals for multiple users. Each request specifies target amount, deadline, initial contribution, priority, lock duration, penalty BPS, and expiration. Returns `BatchGoalResult` with per-goal success/failure details and aggregate metrics.

#### `batch_mark_milestones`

```rust
pub fn batch_mark_milestones(
    env: Env,
    caller: Address,                          // Goal owner
    requests: Vec<MilestoneAchievementRequest>,
) -> BatchMilestoneResult
```

Marks milestones (25%, 50%, 75%, 100%) for multiple goals. Only the goal owner can mark milestones. Validates progress percentage before marking.

---

### Individual Operations

#### `contribute_to_goal`

```rust
pub fn contribute_to_goal(env: Env, caller: Address, goal_id: u64, amount: i128) -> u64
```

Contributes funds to a goal. Enforces prerequisite completion, expiration, and caps at target amount. Returns a `contrib_id` for reversal. Emits milestone events at 25/50/75/100%.

#### `reverse_contribution`

```rust
pub fn reverse_contribution(env: Env, caller: Address, goal_id: u64, contrib_id: u64) -> i128
```

Reverses a contribution within `REVERSAL_PERIOD_SECS` (24 hours). Returns the updated goal balance.

#### `withdraw_from_goal`

```rust
pub fn withdraw_from_goal(env: Env, caller: Address, goal_id: u64, amount: i128) -> i128
```

Withdraws funds from a goal. Blocked if goal is locked (before `unlock_at`). Applies early-withdrawal penalty if `penalty_bps > 0`.

#### `clone_savings_goal`

```rust
pub fn clone_savings_goal(env: Env, caller: Address, goal_id: u64, new_goal_name: Symbol) -> u64
```

Clones a goal with the same target, deadline, priority, lock/expiration durations, and penalty — but with a zero balance and new name.

---

### Read / Query Functions

| Function                           | Returns                  | Description                              |
| ---------------------------------- | ------------------------ | ---------------------------------------- |
| `get_savings_goal(goal_id)`        | `Option<SavingsGoal>`   | Look up a goal by ID                     |
| `get_user_goals(user)`             | `Vec<u64>`               | All goal IDs for a user                  |
| `get_goal_progress(goal_id)`       | `SavingsGoalProgress`   | Progress % and completion status         |
| `get_goal_milestones(goal_id)`     | `Vec<u64>`               | Milestone IDs for a goal                 |
| `get_milestone(milestone_id)`      | `Option<MilestoneAchievement>` | Look up a milestone by ID          |
| `get_total_goals_created()`        | `u64`                    | Lifetime goals created                   |
| `get_total_batches_processed()`    | `u64`                    | Lifetime batches processed               |
| `get_admin()`                      | `Address`                | Current admin address                    |
| `get_goal_snapshots(goal_id)`      | `Vec<GoalSnapshot>`     | Historical balance snapshots             |
| `get_goal_prerequisites(goal_id)`  | `Vec<u64>`               | Prerequisite goal IDs                    |

---

## Storage Layout

### Instance Storage

| DataKey Variant            | Value Type | Description                          |
| -------------------------- | ---------- | ------------------------------------ |
| `Admin`                    | `Address`  | Contract admin                       |
| `LastBatchId`              | `u64`      | Latest batch operation ID            |
| `LastGoalId`               | `u64`      | Latest goal auto-increment ID        |
| `TotalGoalsCreated`        | `u64`      | Lifetime count of goals created      |
| `TotalBatchesProcessed`    | `u64`      | Lifetime count of batches processed  |
| `LastMilestoneId`          | `u64`      | Latest milestone auto-increment ID   |
| `TotalMilestonesAchieved`  | `u64`      | Lifetime milestones achieved         |

### Persistent Storage

| DataKey Variant                    | Value Type               | Description                           |
| ---------------------------------- | ------------------------ | ------------------------------------- |
| `Goal(u64)`                        | `SavingsGoal`            | Goal record by ID                     |
| `UserGoals(Address)`               | `Vec<u64>`               | Goal IDs per user                     |
| `GoalByName(Address, Symbol)`      | `u64`                    | Goal ID lookup by (user, name)        |
| `Milestone(u64)`                   | `MilestoneAchievement`   | Milestone record by ID               |
| `GoalMilestones(u64)`              | `Vec<u64>`               | Milestone IDs per goal                |
| `GoalMilestonesPercent(u64)`       | `Vec<u32>`               | Triggered milestone percentages       |
| `GoalPrereqs(u64)`                 | `Vec<u64>`               | Prerequisite goal IDs                 |
| `GoalClosedAt(u64)`                | `u64`                    | Ledger sequence at auto-close         |
| `GoalSnapshots(u64)`               | `Vec<GoalSnapshot>`      | Historical snapshots                  |
| `Contribution(u64, u64)`           | `ContributionRecord`     | Contribution record (goal, seq)       |
| `LastContribId(u64)`               | `u64`                    | Last contribution index per goal      |

---

## Events

| Event Name               | Topics                                       | Payload                                |
| ------------------------ | -------------------------------------------- | -------------------------------------- |
| `batch_started`          | `("batch", "started")`                       | `(batch_id, request_count)`            |
| `goal_created`           | `("goal", "created", batch_id)`              | `(goal_id, user, target_amount)`       |
| `goal_creation_failed`   | `("goal", "failed", batch_id)`               | `(user, error_code)`                   |
| `batch_completed`        | `("batch", "completed", batch_id)`           | `(successful, failed, total_amount)`   |
| `high_value_goal`        | `("goal", "highval", batch_id)`              | `(goal_id, amount)`                    |
| `milestone_auto`         | `("milestone", "auto", goal_id)`             | `(goal_id, milestone_percent)`         |
| `goal_contributed`       | `("goal", "contrib", goal_id)`               | `(user, amount, new_total)`            |
| `goal_withdrawn`         | `("goal", "withdraw", goal_id)`              | `(user, amount, remaining)`            |
| `goal_withdraw_locked`   | `("goal", "wd_lock", goal_id)`               | `(user, unlock_at)`                    |
| `goal_closed`            | `("goal", "closed", goal_id)`                | `(goal_id, user, final_amount, ledger)`|
| `goal_completed`         | `("goal", "completed", goal_id, user)`       | `(target_amount)`                      |
| `goal_renamed`           | `("goal", "renamed", goal_id)`               | `(old_name, new_name)`                 |
| `goal_snapshot_recorded` | `("goal", "snapshot", goal_id)`              | `(goal_id, amount, timestamp)`         |

---

## Error Codes

| Code | Name                 | Description                                          |
| ---- | -------------------- | ---------------------------------------------------- |
| 1    | `NotInitialized`     | Contract has not been initialized                    |
| 2    | `Unauthorized`       | Caller is not admin or goal owner                    |
| 3    | `InvalidBatch`       | Batch request data is invalid                        |
| 4    | `EmptyBatch`         | Batch is empty                                       |
| 5    | `BatchTooLarge`      | Batch exceeds `MAX_BATCH_SIZE` (100)                 |
| 6    | `GoalClosed`         | Goal target reached; no more contributions accepted  |
| 7    | `InsufficientBalance`| Withdrawal exceeds current balance                   |
| 8    | `GoalNotActive`      | Goal is not in an active state                       |
| 9    | `InvalidGoalName`    | Goal name is empty or duplicate                      |
| 10   | `InvalidAmount`      | Contribution/withdrawal amount invalid               |
| 11   | `GoalNotFound`       | No goal exists with the given ID                     |
| 12   | `GoalLocked`         | Withdrawals blocked before unlock time               |
| 13   | `GoalExpired`        | Goal has passed its expiration timestamp              |
| 14   | `DependencyNotMet`   | Prerequisite goals are not yet complete               |
| 15   | `ReversalExpired`    | 24-hour reversal window has elapsed                   |
| 16   | `ContributionNotFound`| Contribution ID not found or already reversed        |

---

## Types

### `SavingsGoal`

```rust
pub struct SavingsGoal {
    pub goal_id: u64,
    pub user: Address,
    pub goal_name: Symbol,
    pub target_amount: i128,     // in stroops
    pub current_amount: i128,    // in stroops
    pub deadline: u64,
    pub created_at: u64,
    pub is_active: bool,
    pub is_complete: bool,
    pub priority: u32,
    pub unlock_at: u64,          // 0 = no lock
    pub expires_at: u64,         // 0 = no expiration
    pub penalty_bps: u32,        // early withdrawal penalty
}
```

### `SavingsGoalRequest`

```rust
pub struct SavingsGoalRequest {
    pub user: Address,
    pub goal_name: Symbol,
    pub target_amount: i128,
    pub deadline: u64,
    pub initial_contribution: i128,
    pub priority: u32,
    pub lock_duration_seconds: u64,
    pub penalty_bps: u32,
    pub expiration_seconds: u64,
}
```

### Constants

| Constant              | Value                     | Description                    |
| --------------------- | ------------------------- | ------------------------------ |
| `MAX_BATCH_SIZE`      | `100`                     | Max items per batch            |
| `MIN_GOAL_AMOUNT`     | `10_000_000`              | 1 XLM in stroops               |
| `MAX_GOAL_AMOUNT`     | `1_000_000_000_000_000_000`| 1 billion XLM in stroops      |
| `REVERSAL_PERIOD_SECS`| `86_400`                  | 24-hour reversal window        |
| `PERSISTENT_TTL_BUMP` | `12_614_400`              | ~146 days TTL bump             |

---

## Usage Example

```rust
use soroban_sdk::{Env, Address, Symbol, Vec};
use savings_goals::{SavingsGoalsContract, SavingsGoalRequest};

// 1. Initialize
contract.initialize(&env, &admin);

// 2. Batch create goals
let result = contract.batch_set_savings_goals(&env, &admin, &vec![
    &env,
    SavingsGoalRequest {
        user: user_a.clone(),
        goal_name: Symbol::new(&env, "vacation"),
        target_amount: 50_000_000_000,  // 5000 XLM
        deadline: 1_000_000,
        initial_contribution: 1_000_000_000,
        priority: 1,
        lock_duration_seconds: 0,
        penalty_bps: 0,
        expiration_seconds: 0,
    },
]);
assert_eq!(result.successful, 1);

// 3. Contribute
let contrib_id = contract.contribute_to_goal(&env, &user_a, 1, 5_000_000_000);

// 4. Check progress
let progress = contract.get_goal_progress(&env, 1);
```

---

## Testing

```bash
cargo test -p savings-goals
```

---

## Design Notes

- **Single-pass O(n) batch processing**: Each batch request is processed in a single loop to minimize compute overhead.
- **Contribution capping**: Contributions are capped at the remaining amount to reach the target, preventing over-accumulation.
- **Reversal window**: The `REVERSAL_PERIOD_SECS` (24h) allows users to undo accidental contributions. After the window, contributions are permanent.
- **Prerequisite graph**: Goals can declare prerequisites via `GoalPrereqs`. Contributions are blocked until all prerequisites are marked complete. This enables sequential savings workflows (e.g., "emergency fund before vacation fund").
- **Name uniqueness**: `GoalByName(Address, Symbol)` enforces that a user cannot have two goals with the same name.
- **`PERSISTENT_TTL_BUMP`**: Goal records get a ~146-day TTL bump on access, balancing storage rent costs against goal lifecycle duration.

---

## License

MIT
