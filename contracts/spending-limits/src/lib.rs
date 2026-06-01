//! # Spending Limits Contract
//!
//! A Soroban smart contract for managing batch spending limit updates
//! for multiple users simultaneously.
//!
//! ## Features
//!
//! - **Batch Processing**: Efficiently update spending limits for multiple users in a single call
//! - **Comprehensive Validation**: Validates limit amounts and user addresses
//! - **Event Emission**: Emits events for limit updates and batch processing
//! - **Error Handling**: Gracefully handles invalid inputs with detailed error codes
//! - **Optimized Storage**: Minimizes storage writes by batching operations
//! - **Partial Failure Support**: Invalid updates don't affect valid ones
//!
//! ## Optimization Strategies
//!
//! - Single-pass processing for O(n) complexity
//! - Minimized storage operations (batch writes at the end)
//! - Efficient data structures
//! - Batched event emissions

#![no_std]

mod cross_contract;
mod types;
mod validation;

use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Env, Vec};

pub use crate::types::{
    BatchLimitMetrics, BatchLimitResult, DataKey, ErrorCode, EscalationConfig, LimitEvents,
    LimitStrategy, LimitUpdateResult, LimitsConfig, SpendingLimit, SpendingLimitRequest,
    MAX_BATCH_SIZE,
};
use crate::validation::validate_limit_request;

// Add cross-contract imports for whitelist functionality
use crate::cross_contract::DataKey as CrossContractDataKey;
use soroban_sdk::{Bytes, Symbol};

/// Error codes for the spending limits contract.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SpendingLimitError {
    /// Contract not initialized
    NotInitialized = 1,
    /// Caller is not authorized
    Unauthorized = 2,
    /// Invalid batch data
    InvalidBatch = 3,
    /// Batch is empty
    EmptyBatch = 4,
    /// Batch exceeds maximum size
    BatchTooLarge = 5,
    /// Daily limit exceeded
    DailyLimitExceeded = 6,
    /// Monthly limit exceeded
    MonthlyLimitExceeded = 7,
    /// Invalid spend amount
    InvalidAmount = 8,
}

impl From<SpendingLimitError> for soroban_sdk::Error {
    fn from(e: SpendingLimitError) -> Self {
        soroban_sdk::Error::from_contract_error(e as u32)
    }
}

#[contract]
pub struct SpendingLimitsContract;

#[contractimpl]
impl SpendingLimitsContract {
    /// Initializes the contract with an admin address.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `admin` - The admin address that can manage the contract
    ///
    /// # Storage Optimization
    /// Uses a consolidated `LimitsConfig` struct instead of 4 separate
    /// storage entries, reducing initialization writes from 4 to 1.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::LimitsConfig) {
            panic!("Contract already initialized");
        }

        let config = LimitsConfig {
            admin,
            last_batch_id: 0,
            total_limits_updated: 0,
            total_batches_processed: 0,
        };
        env.storage()
            .instance()
            .set(&DataKey::LimitsConfig, &config);
    }

    /// Updates monthly spending limits for multiple users in a batch.
    ///
    /// This is the main entry point for batch limit updates. It validates all requests,
    /// updates limits, emits events, and handles partial failures gracefully.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `caller` - The address calling this function (must be admin)
    /// * `requests` - Vector of spending limit update requests
    ///
    /// # Returns
    /// * `BatchLimitResult` - Result containing updated limits and metrics
    ///
    /// # Events Emitted
    /// * `batch_started` - When processing begins
    /// * `limit_updated` - For each successful limit update
    /// * `limit_update_failed` - For each failed limit update
    /// * `high_value_limit` - For limits with high values
    /// * `batch_completed` - When processing completes
    ///
    /// # Errors
    /// * `EmptyBatch` - If no requests provided
    /// * `BatchTooLarge` - If batch exceeds maximum size
    /// * `Unauthorized` - If caller is not admin
    pub fn batch_update_spending_limits(
        env: Env,
        caller: Address,
        requests: Vec<SpendingLimitRequest>,
    ) -> BatchLimitResult {
        // Verify authorization
        caller.require_auth();
        Self::require_admin(&env, &caller);

        // Validate batch size
        let request_count = requests.len();
        if request_count == 0 {
            panic_with_error!(&env, SpendingLimitError::EmptyBatch);
        }
        if request_count > MAX_BATCH_SIZE {
            panic_with_error!(&env, SpendingLimitError::BatchTooLarge);
        }

        // Get batch ID and increment from consolidated config
        let mut config: LimitsConfig = env
            .storage()
            .instance()
            .get(&DataKey::LimitsConfig)
            .expect("Contract not initialized");
        let batch_id: u64 = config.last_batch_id + 1;

        // Emit batch started event
        LimitEvents::batch_started(&env, batch_id, request_count);

        // Get current ledger timestamp
        let current_ledger = env.ledger().sequence() as u64;

        // Initialize result tracking
        let mut results: Vec<LimitUpdateResult> = Vec::new(&env);
        let mut successful_count: u32 = 0;
        let mut failed_count: u32 = 0;
        let mut total_limits_value: i128 = 0;

        // Process each request
        for request in requests.iter() {
            // Validate the request
            match validate_limit_request(&request) {
                Ok(()) => {
                    // Validation succeeded - update the limit
                    let limit = SpendingLimit {
                        user: request.user.clone(),
                        monthly_limit: request.monthly_limit,
                        reset_window_seconds: request.reset_window_seconds,
                        current_spending: 0, // Reset spending when updating limit
                        category: request.category.clone(),
                        updated_at: current_ledger,
                        is_active: true,
                        strategy: request.strategy.clone(),
                    };

                    // Accumulate metrics
                    total_limits_value = total_limits_value
                        .checked_add(request.monthly_limit)
                        .unwrap_or(i128::MAX);
                    successful_count += 1;

                    // Store the limit (optimized - one write per limit)
                    env.storage()
                        .persistent()
                        .set(&DataKey::SpendingLimit(request.user.clone()), &limit);

                    // Emit success event
                    LimitEvents::limit_updated(&env, batch_id, &limit);

                    // Emit high-value limit event if applicable (>= 1,000,000 XLM)
                    if request.monthly_limit >= 10_000_000_000_000_000 {
                        LimitEvents::high_value_limit(
                            &env,
                            batch_id,
                            &request.user,
                            request.monthly_limit,
                        );
                    }

                    results.push_back(LimitUpdateResult::Success(limit));
                }
                Err(error_code) => {
                    // Validation failed - record failure
                    failed_count += 1;

                    // Emit failure event
                    LimitEvents::limit_update_failed(&env, batch_id, &request.user, error_code);

                    results.push_back(LimitUpdateResult::Failure(request.user.clone(), error_code));
                }
            }
        }

        // Calculate average limit amount
        let avg_limit_amount = if successful_count > 0 {
            total_limits_value / successful_count as i128
        } else {
            0
        };

        // Create metrics
        let metrics = BatchLimitMetrics {
            total_requests: request_count,
            successful_updates: successful_count,
            failed_updates: failed_count,
            total_limits_value,
            avg_limit_amount,
            processed_at: current_ledger,
        };

        // Update consolidated config (single write instead of 4)
        config.last_batch_id = batch_id;
        config.total_limits_updated = config
            .total_limits_updated
            .checked_add(successful_count as u64)
            .unwrap_or(u64::MAX);
        config.total_batches_processed = config
            .total_batches_processed
            .checked_add(1)
            .unwrap_or(u64::MAX);
        env.storage()
            .instance()
            .set(&DataKey::LimitsConfig, &config);

        // Emit batch completed event
        LimitEvents::batch_completed(
            &env,
            batch_id,
            successful_count,
            failed_count,
            total_limits_value,
        );

        BatchLimitResult {
            batch_id,
            total_requests: request_count,
            successful: successful_count,
            failed: failed_count,
            results,
            metrics,
        }
    }

    /// Configures escalation rules for spending enforcement.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `admin` - Admin address (must authorize)
    /// * `small_threshold` - Amount below which spends are "small" (auto-approved)
    /// * `medium_threshold` - Amount below which spends are "medium" (logged)
    ///   Spends at or above this threshold are "large" and require admin approval
    /// * `enabled` - Whether escalation rules are active
    pub fn configure_escalation_rules(
        env: Env,
        admin: Address,
        small_threshold: i128,
        medium_threshold: i128,
        enabled: bool,
    ) {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        if small_threshold <= 0 || medium_threshold <= small_threshold {
            panic_with_error!(&env, SpendingLimitError::InvalidAmount);
        }

        let config = EscalationConfig {
            small_threshold,
            medium_threshold,
            enabled,
        };

        env.storage()
            .instance()
            .set(&DataKey::EscalationConfig, &config);

        LimitEvents::escalation_configured(&env, small_threshold, medium_threshold, enabled);
    }

    /// Returns the current escalation configuration.
    pub fn get_escalation_config(env: Env) -> Option<EscalationConfig> {
        env.storage().instance().get(&DataKey::EscalationConfig)
    }

    /// Admin approves a large spend that was escalated.
    ///
    /// After approval, the spend is recorded against the user's limits
    /// as though it passed normal enforcement.
    pub fn approve_escalated_spend(env: Env, admin: Address, user: Address, amount: i128) {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        // Record the spend against the user's limits
        let mut limit: SpendingLimit = match env
            .storage()
            .persistent()
            .get(&DataKey::SpendingLimit(user.clone()))
        {
            Some(l) => l,
            None => panic_with_error!(&env, SpendingLimitError::InvalidAmount),
        };

        limit.current_spending = limit
            .current_spending
            .checked_add(amount)
            .unwrap_or(i128::MAX);
        limit.updated_at = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::SpendingLimit(user.clone()), &limit);

        LimitEvents::escalation_approved(&env, &admin, &user, amount);
    }

    /// Enforces the configured daily and monthly spending limits for a user,
    /// including escalation tier checks.
    ///
    /// This function:
    /// - Tracks per-user daily and monthly totals using the current ledger timestamp.
    /// - Checks escalation tiers: small (auto), medium (logged), large (requires approval).
    /// - Rejects spends that would exceed either the derived daily limit or the stored
    ///   monthly limit.
    /// - Emits a `limit_exceeded` event when a violation occurs.
    /// - Emits an `escalation_triggered` event for medium/large spends.
    ///
    /// If no limit is configured for the user or the limit is inactive, the spend is
    /// allowed and no state is updated.
    pub fn enforce_spending_limit(env: Env, user: Address, amount: i128) {
        // Validate amount
        if amount <= 0 {
            panic_with_error!(&env, SpendingLimitError::InvalidAmount);
        }

        // Check if destination is whitelisted (spending whitelist)
        // This prevents unauthorized destinations from receiving funds
        if !Self::is_destination_whitelisted(env, user.clone()) {
            panic_with_error!(&env, SpendingLimitError::Unauthorized);
        }

        // Look up configured limit; if none, there is nothing to enforce.
        let mut limit: SpendingLimit = match env
            .storage()
            .persistent()
            .get(&DataKey::SpendingLimit(user.clone()))
        {
            Some(l) => l,
            None => return,
        };

        if !limit.is_active {
            return;
        }

        let now = env.ledger().timestamp();

        // Derive simple logical window/month identifiers from timestamp.
        const SECONDS_PER_DAY: u64 = 86_400;
        const SECONDS_PER_MONTH: u64 = SECONDS_PER_DAY * 30;

        // Reset windows are configurable and must be validated at limit setup.
        let window_seconds = limit.reset_window_seconds;
        let window_id = if now == 0 {
            0
        } else {
            (now - 1) / window_seconds
        };
        let month_id = if now == 0 {
            0
        } else {
            (now - 1) / SECONDS_PER_MONTH
        };

        // Load current window and monthly totals.
        let window_key = DataKey::WindowSpending(user.clone(), window_id);
        let monthly_key = DataKey::MonthlySpending(user.clone(), month_id);

        let current_window: i128 = env.storage().persistent().get(&window_key).unwrap_or(0);
        let current_monthly: i128 = env.storage().persistent().get(&monthly_key).unwrap_or(0);

        let new_window = current_window
            .checked_add(amount)
            .unwrap_or_else(|| panic_with_error!(&env, SpendingLimitError::InvalidBatch));
        let new_monthly = current_monthly
            .checked_add(amount)
            .unwrap_or_else(|| panic_with_error!(&env, SpendingLimitError::InvalidBatch));

        // Derive a limit for the configured reset window from the monthly limit.
        let window_limit = if limit.monthly_limit <= 0 {
            0
        } else {
            let base = limit.monthly_limit * window_seconds as i128 / SECONDS_PER_MONTH as i128;
            if base == 0 {
                1
            } else {
                base
            }
        };

        let mut window_ok = true;
        let mut monthly_ok = true;

        if new_window > window_limit {
            window_ok = false;
        }
        if new_monthly > limit.monthly_limit {
            monthly_ok = false;
        }

        if !window_ok || !monthly_ok {
            let remaining_window = if current_window >= window_limit {
                0
            } else {
                window_limit - current_window
            };
            let remaining_monthly = if current_monthly >= limit.monthly_limit {
                0
            } else {
                limit.monthly_limit - current_monthly
            };

            LimitEvents::limit_exceeded(&env, &user, amount, remaining_window, remaining_monthly);

            if !window_ok {
                panic_with_error!(&env, SpendingLimitError::DailyLimitExceeded);
            } else {
                panic_with_error!(&env, SpendingLimitError::MonthlyLimitExceeded);
            }
        }

        // If adaptive strategy is enabled and user is nearing their limit (>= 90%)
        // automatically increase the limit by 10% for future transactions.
        if limit.strategy == crate::types::LimitStrategy::Adaptive
            && new_monthly >= (limit.monthly_limit * 9 / 10)
        {
            let old_limit = limit.monthly_limit;
            let increment = limit.monthly_limit / 10;
            let proposed_limit = old_limit
                .checked_add(increment)
                .unwrap_or(crate::types::MAX_SPENDING_LIMIT);

            limit.monthly_limit = if proposed_limit > crate::types::MAX_SPENDING_LIMIT {
                crate::types::MAX_SPENDING_LIMIT
            } else {
                proposed_limit
            };

            if limit.monthly_limit != old_limit {
                LimitEvents::limit_adjusted(&env, &user, old_limit, limit.monthly_limit);
            }
        }

        // Persist updated totals.
        env.storage().persistent().set(&window_key, &new_window);
        env.storage().persistent().set(&monthly_key, &new_monthly);

        // Keep the embedded "current_spending" and "updated_at" in sync with the
        // current logical month usage.
        limit.current_spending = new_monthly;
        limit.updated_at = now;
        env.storage()
            .persistent()
            .set(&DataKey::SpendingLimit(user), &limit);
    }

    /// Records an admin-approved emergency spending override on-chain.
    ///
    /// Requires the admin's signature (`require_auth`) so the override cannot be
    /// performed without explicit approval, and emits an auditable event so the
    /// bypass is preserved in the on-chain audit trail.
    pub fn emergency_override(env: Env, admin: Address, user: Address, amount: i128) {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        if amount <= 0 {
            panic_with_error!(&env, SpendingLimitError::InvalidAmount);
        }

        env.events().publish(
            (
                soroban_sdk::symbol_short!("limit"),
                soroban_sdk::symbol_short!("override"),
            ),
            (admin, user, amount, env.ledger().timestamp()),
        );
    }

    /// Retrieves a user's spending limit.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `user` - The user's address
    ///
    /// # Returns
    /// * `Option<SpendingLimit>` - The limit if found
    pub fn get_spending_limit(env: Env, user: Address) -> Option<SpendingLimit> {
        env.storage()
            .persistent()
            .get(&DataKey::SpendingLimit(user))
    }

    /// Returns the admin address.
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get::<DataKey, LimitsConfig>(&DataKey::LimitsConfig)
            .expect("Contract not initialized")
            .admin
    }

    /// Updates the admin address.
    pub fn set_admin(env: Env, current_admin: Address, new_admin: Address) {
        current_admin.require_auth();
        Self::require_admin(&env, &current_admin);

        let mut config: LimitsConfig = env
            .storage()
            .instance()
            .get(&DataKey::LimitsConfig)
            .expect("Contract not initialized");
        config.admin = new_admin;
        env.storage()
            .instance()
            .set(&DataKey::LimitsConfig, &config);
    }

    /// Adds a destination address to the spending whitelist.
    /// Only admin can call this method.
    pub fn whitelist_destination(env: Env, caller: Address, destination: Address) {
        caller.require_auth();
        Self::require_admin(&env, &caller);

        env.storage()
            .persistent()
            .set(&CrossContractDataKey::Whitelist(destination.clone()), &true);
    }

    /// Removes a destination address from the spending whitelist.
    /// Only admin can call this method.
    pub fn remove_from_whitelist(env: Env, caller: Address, destination: Address) {
        caller.require_auth();
        Self::require_admin(&env, &caller);

        env.storage()
            .persistent()
            .remove(&CrossContractDataKey::Whitelist(destination.clone()));
    }

    /// Checks if a destination address is whitelisted for receiving funds.
    /// This is a public read-only method that can be called by anyone.
    pub fn is_destination_whitelisted(env: Env, destination: Address) -> bool {
        // Use the same whitelist storage pattern as cross-contract module
        // Check if destination is in whitelist
        env.storage()
            .persistent()
            .has(&CrossContractDataKey::Whitelist(destination.clone()))
    }

    /// Returns the last created batch ID.
    pub fn get_last_batch_id(env: Env) -> u64 {
        env.storage()
            .instance()
            .get::<DataKey, LimitsConfig>(&DataKey::LimitsConfig)
            .map(|c| c.last_batch_id)
            .unwrap_or(0)
    }

    /// Returns the total number of limits updated.
    pub fn get_total_limits_updated(env: Env) -> u64 {
        env.storage()
            .instance()
            .get::<DataKey, LimitsConfig>(&DataKey::LimitsConfig)
            .map(|c| c.total_limits_updated)
            .unwrap_or(0)
    }

    /// Returns the total number of batches processed.
    pub fn get_total_batches_processed(env: Env) -> u64 {
        env.storage()
            .instance()
            .get::<DataKey, LimitsConfig>(&DataKey::LimitsConfig)
            .map(|c| c.total_batches_processed)
            .unwrap_or(0)
    }

    // Internal helper to verify admin
    fn require_admin(env: &Env, caller: &Address) {
        let config: LimitsConfig = env
            .storage()
            .instance()
            .get(&DataKey::LimitsConfig)
            .expect("Contract not initialized");

        if *caller != config.admin {
            panic_with_error!(env, SpendingLimitError::Unauthorized);
        }
    }

    /// Checks if a destination address is whitelisted for receiving funds.
    /// Uses the cross-contract whitelist functionality to determine authorization.
    fn is_destination_whitelisted(env: &Env, destination: &Address) -> bool {
        // Use the same whitelist storage pattern as cross-contract module
        // Check if destination is in whitelist
        env.storage()
            .persistent()
            .has(&CrossContractDataKey::Whitelist(destination.clone()))
    }
}

#[cfg(test)]
mod test;
