//! Persistent storage helpers for the rewards contract.
//!
//! All reward data is stored in **persistent** storage so that balances survive
//! ledger state expiry. Each helper follows the read-modify-write pattern and
//! bumps the TTL on every access to keep entries alive.

use soroban_sdk::{Address, Env};

use crate::types::{DataKey, RewardAccount, RewardTransaction, PERSISTENT_TTL_BUMP};

// ── Reward Balance ─────────────────────────────────────────────────────────────

/// Returns the current claimable reward balance for `account` (stroops).
///
/// Returns `0` if no entry exists yet.
pub fn get_reward_balance(env: &Env, account: &Address) -> i128 {
    let key = DataKey::RewardBalance(account.clone());
    let balance = env
        .storage()
        .persistent()
        .get::<DataKey, i128>(&key)
        .unwrap_or(0);
    if balance != 0 {
        env.storage()
            .persistent()
            .extend_ttl(&key, PERSISTENT_TTL_BUMP, PERSISTENT_TTL_BUMP);
    }
    balance
}

/// Overwrites the claimable reward balance for `account`.
pub fn set_reward_balance(env: &Env, account: &Address, balance: i128) {
    let key = DataKey::RewardBalance(account.clone());
    env.storage().persistent().set(&key, &balance);
    env.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_BUMP, PERSISTENT_TTL_BUMP);
}

// ── Lifetime Earned ────────────────────────────────────────────────────────────

/// Returns the total rewards ever earned by `account` (stroops).
///
/// Returns `0` if no entry exists yet.
pub fn get_lifetime_earned(env: &Env, account: &Address) -> i128 {
    let key = DataKey::LifetimeEarned(account.clone());
    let earned = env
        .storage()
        .persistent()
        .get::<DataKey, i128>(&key)
        .unwrap_or(0);
    if earned != 0 {
        env.storage()
            .persistent()
            .extend_ttl(&key, PERSISTENT_TTL_BUMP, PERSISTENT_TTL_BUMP);
    }
    earned
}

/// Overwrites the lifetime-earned total for `account`.
pub fn set_lifetime_earned(env: &Env, account: &Address, amount: i128) {
    let key = DataKey::LifetimeEarned(account.clone());
    env.storage().persistent().set(&key, &amount);
    env.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_BUMP, PERSISTENT_TTL_BUMP);
}

// ── Lifetime Claimed ───────────────────────────────────────────────────────────

/// Returns the total rewards ever claimed by `account` (stroops).
///
/// Returns `0` if no entry exists yet.
pub fn get_lifetime_claimed(env: &Env, account: &Address) -> i128 {
    let key = DataKey::LifetimeClaimed(account.clone());
    let claimed = env
        .storage()
        .persistent()
        .get::<DataKey, i128>(&key)
        .unwrap_or(0);
    if claimed != 0 {
        env.storage()
            .persistent()
            .extend_ttl(&key, PERSISTENT_TTL_BUMP, PERSISTENT_TTL_BUMP);
    }
    claimed
}

/// Overwrites the lifetime-claimed total for `account`.
pub fn set_lifetime_claimed(env: &Env, account: &Address, amount: i128) {
    let key = DataKey::LifetimeClaimed(account.clone());
    env.storage().persistent().set(&key, &amount);
    env.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_BUMP, PERSISTENT_TTL_BUMP);
}

// ── Reward Account Metadata ────────────────────────────────────────────────────

/// Returns the full `RewardAccount` metadata for `account`, if it exists.
pub fn get_reward_account(env: &Env, account: &Address) -> Option<RewardAccount> {
    let key = DataKey::RewardAccount(account.clone());
    let result = env
        .storage()
        .persistent()
        .get::<DataKey, RewardAccount>(&key);
    if result.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, PERSISTENT_TTL_BUMP, PERSISTENT_TTL_BUMP);
    }
    result
}

/// Persists a `RewardAccount` metadata record.
pub fn set_reward_account(env: &Env, account: &Address, record: &RewardAccount) {
    let key = DataKey::RewardAccount(account.clone());
    env.storage().persistent().set(&key, record);
    env.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_BUMP, PERSISTENT_TTL_BUMP);
}

/// Returns `true` if a reward account record exists for `account`.
pub fn has_reward_account(env: &Env, account: &Address) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::RewardAccount(account.clone()))
}

// ── Reward Transaction Counter ─────────────────────────────────────────────────

/// Returns the next available reward transaction ID.
///
/// Returns `0` if no transactions have been created yet.
pub fn get_reward_tx_counter(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get::<DataKey, u64>(&DataKey::RewardTxCounter)
        .unwrap_or(0)
}

/// Persists the reward transaction counter.
pub fn set_reward_tx_counter(env: &Env, counter: u64) {
    env.storage()
        .instance()
        .set(&DataKey::RewardTxCounter, &counter);
}

// ── Reward Transaction Records ─────────────────────────────────────────────────

/// Returns the `RewardTransaction` record for the given `id`, if it exists.
pub fn get_reward_transaction(env: &Env, id: u64) -> Option<RewardTransaction> {
    let key = DataKey::RewardTransaction(id);
    let result = env
        .storage()
        .persistent()
        .get::<DataKey, RewardTransaction>(&key);
    if result.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, PERSISTENT_TTL_BUMP, PERSISTENT_TTL_BUMP);
    }
    result
}

/// Persists a `RewardTransaction` record keyed by its `id`.
pub fn set_reward_transaction(env: &Env, id: u64, tx: &RewardTransaction) {
    let key = DataKey::RewardTransaction(id);
    env.storage().persistent().set(&key, tx);
    env.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_BUMP, PERSISTENT_TTL_BUMP);
}
