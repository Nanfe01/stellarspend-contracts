use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol, Vec};

/// Maximum number of requests in a single batch for optimization.
pub const MAX_BATCH_SIZE: u32 = 100;

/// Minimum spending limit (1 XLM in stroops).
pub const MIN_SPENDING_LIMIT: i128 = 10_000_000;

/// Maximum spending limit (1 billion XLM in stroops).
pub const MAX_SPENDING_LIMIT: i128 = 1_000_000_000_000_000_000;

/// Minimum reset window in seconds (1 hour).
pub const MIN_RESET_WINDOW_SECONDS: u64 = 3_600;

/// Maximum reset window in seconds (90 days).
pub const MAX_RESET_WINDOW_SECONDS: u64 = 7_776_000;

/// Escalation levels for spending limit enforcement.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum EscalationLevel {
    /// Small spend — automatic approval
    Small,
    /// Medium spend — logged but automatically approved
    Medium,
    /// Large spend — requires admin approval
    Large,
}

/// Strategies for spending limit adjustment.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum LimitStrategy {
    /// Fixed monthly limit
    Static,
    /// Limit increases automatically based on usage
    Adaptive,
}

/// Configuration for spending escalation rules.
#[derive(Clone, Debug)]
#[contracttype]
pub struct EscalationConfig {
    /// Threshold for small-to-medium escalation (in stroops)
    pub small_threshold: i128,
    /// Threshold for medium-to-large escalation (in stroops)
    pub medium_threshold: i128,
    /// Whether escalation rules are enabled
    pub enabled: bool,
}

/// Represents a spending limit request for a user.
#[derive(Clone, Debug)]
#[contracttype]
pub struct SpendingLimitRequest {
    /// User's address
    pub user: Address,
    /// Monthly spending limit amount (in stroops)
    pub monthly_limit: i128,
    /// New daily spending limit (in stroops)
    pub daily_limit: i128,
    /// New hourly spending limit (in stroops)
    pub hourly_limit: i128,
    /// Reset window for the spending limit (in seconds)
    /// Reset window in seconds (e.g., 86400 for daily)
    pub reset_window_seconds: u64,
    /// Optional spending category
    pub category: Option<Symbol>,
    /// Adjustment strategy
    pub strategy: LimitStrategy,
}

/// Represents a configured spending limit for a user.
#[derive(Clone, Debug)]
#[contracttype]
pub struct SpendingLimit {
    /// User's address
    pub user: Address,
    /// Monthly spending limit amount (in stroops)
    pub monthly_limit: i128,

    /// Daily spending limit (in stroops)
    pub daily_limit: i128,
    /// Hourly spending limit (in stroops)
    pub hourly_limit: i128,
   

    /// Reset window in seconds

    pub reset_window_seconds: u64,
    /// Current spending tracked in this period
    pub current_spending: i128,
    /// Optional category for the limit
    pub category: Option<Symbol>,
    /// When the limit was last updated (ledger timestamp)
    pub updated_at: u64,
    /// Whether the limit is active
    pub is_active: bool,
    /// Adjustment strategy
    pub strategy: LimitStrategy,
}

/// Consolidated instance-storage configuration for the spending limits contract.
#[derive(Clone)]
#[contracttype]
pub struct LimitsConfig {
    pub admin: Address,
    pub last_batch_id: u64,
    pub total_limits_updated: u64,
    pub total_batches_processed: u64,
}

/// Result of processing a single limit update.
#[derive(Clone, Debug)]
#[contracttype]
pub enum LimitUpdateResult {
    Success(SpendingLimit),
    Failure(Address, u32), // user address, error code
}

/// Aggregated metrics for a batch of limit updates.
#[derive(Clone, Debug)]
#[contracttype]
pub struct BatchLimitMetrics {
    /// Total number of limit update requests
    pub total_requests: u32,
    /// Number of successful updates
    pub successful_updates: u32,
    /// Number of failed updates
    pub failed_updates: u32,
    /// Total value of all limits
    pub total_limits_value: i128,
    /// Average limit amount
    pub avg_limit_amount: i128,
    /// Batch processing timestamp
    pub processed_at: u64,
}

/// Result of batch limit updates.
#[derive(Clone, Debug)]
#[contracttype]
pub struct BatchLimitResult {
    /// Batch ID
    pub batch_id: u64,
    /// Total number of requests
    pub total_requests: u32,
    /// Number of successful updates
    pub successful: u32,
    /// Number of failed updates
    pub failed: u32,
    /// Individual update results
    pub results: Vec<LimitUpdateResult>,
    /// Aggregated metrics
    pub metrics: BatchLimitMetrics,
}

/// Storage keys for contract state.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Consolidated limits configuration.
    LimitsConfig,
    /// Stored spending limit by user address.
    SpendingLimit(Address),
    /// Windowed spending tracking (user, window_id).
    WindowSpending(Address, u64),
    /// Monthly spending tracking (user, month_id).
    MonthlySpending(Address, u64),
    /// Per-user hourly spending for a given logical hour identifier.
    HourlySpending(Address, u64),
    /// Per-user daily spending for a given logical day identifier.
    DailySpending(Address, u64),
    /// Escalation configuration.
    EscalationConfig,
}

/// Error codes for limit validation and enforcement.
pub mod ErrorCode {
    /// Invalid limit amount (negative or zero)
    pub const INVALID_LIMIT: u32 = 0;
    /// Invalid user address
    pub const INVALID_USER_ADDRESS: u32 = 1;
    /// Invalid reset window
    pub const INVALID_RESET_WINDOW: u32 = 2;
    /// Limit not found
    pub const LIMIT_NOT_FOUND: u32 = 3;
}

/// Event helpers for the spending limits contract.
pub struct LimitEvents;

impl LimitEvents {
    pub fn batch_started(env: &Env, batch_id: u64, count: u32) {
        env.events().publish(
            (symbol_short!("limit"), symbol_short!("batch_st")),
            (batch_id, count),
        );
    }

    pub fn limit_updated(env: &Env, batch_id: u64, limit: &SpendingLimit) {
        env.events().publish(
            (symbol_short!("limit"), symbol_short!("updated")),
            (batch_id, limit.user.clone(), limit.monthly_limit),
        );
    }

    pub fn limit_adjusted(env: &Env, user: &Address, old_limit: i128, new_limit: i128) {
        env.events().publish(
            (symbol_short!("limit"), symbol_short!("adjusted")),
            (user.clone(), old_limit, new_limit),
        );
    }

    pub fn high_value_limit(env: &Env, batch_id: u64, user: &Address, amount: i128) {
        env.events().publish(
            (symbol_short!("limit"), symbol_short!("high_val")),
            (batch_id, user.clone(), amount),
        );
    }

    pub fn limit_update_failed(env: &Env, batch_id: u64, user: &Address, error_code: u32) {
        env.events().publish(
            (symbol_short!("limit"), symbol_short!("upd_fail")),
            (batch_id, user.clone(), error_code),
        );
    }

    pub fn batch_completed(env: &Env, batch_id: u64, success: u32, failed: u32, total: i128) {
        env.events().publish(
            (symbol_short!("limit"), symbol_short!("batch_cp")),
            (batch_id, success, failed, total),
        );
    }

    /// Event emitted when a spend attempt exceeds either the hourly, daily, or monthly limit.
    pub fn limit_exceeded(
        env: &Env,
        user: &Address,
        attempted_amount: i128,
        remaining_hourly: i128,
        remaining_daily: i128,
        remaining_monthly: i128,
    ) {
        env.events().publish(
            topics,
            (
                user.clone(),
                attempted_amount,
                remaining_hourly,
                remaining_daily,
                remaining_monthly,
            ),
            (symbol_short!("limit"), symbol_short!("exceeded")),
            (user.clone(), amount, remaining_window, remaining_monthly),
        );
    }

    pub fn escalation_configured(env: &Env, small: i128, medium: i128, enabled: bool) {
        env.events().publish(
            (symbol_short!("limit"), symbol_short!("esc_cfg")),
            (small, medium, enabled),
        );
    }

    pub fn escalation_approved(env: &Env, admin: &Address, user: &Address, amount: i128) {
        env.events().publish(
            (symbol_short!("limit"), symbol_short!("esc_app")),
            (admin.clone(), user.clone(), amount),
        );
    }

    pub fn spending_limit_overridden(
        env: &Env,
        admin: &Address,
        user: &Address,
        old_limit: i128,
        new_limit: i128,
    ) {
        let topics = (
            symbol_short!("limit"),
            symbol_short!("override"),
        );
    
        env.events().publish(
            topics,
            (
                admin.clone(),
                user.clone(),
                old_limit,
                new_limit,
            ),
        );
    }
}
