//! Data types and events for spending limit operations.

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
    /// Reset window in seconds (e.g., 86400 for daily)
    pub reset_window_seconds: u64,
    /// Optional spending category
    pub category: Option<BudgetCategory>,
}

/// Represents a configured spending limit for a user.
#[derive(Clone, Debug)]
#[contracttype]
pub struct SpendingLimit {
    /// User's address
    pub user: Address,
    /// Monthly spending limit amount (in stroops)
    pub monthly_limit: i128,
    /// Reset window in seconds
    pub reset_window_seconds: u64,
    /// Current spending tracked in this period
    pub current_spending: i128,
    /// Optional category for the limit
    pub category: Option<BudgetCategory>,
    /// When the limit was last updated (ledger timestamp)
    pub updated_at: u64,
    /// Whether the limit is active
    pub is_active: bool,
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
    /// Admin address
    Admin,
    /// Last created batch ID
    LastBatchId,
    /// Total limits updated lifetime
    TotalLimitsUpdated,
    /// Total batches processed lifetime
    TotalBatchesProcessed,
    /// Stored spending limit by user address
    SpendingLimit(Address),
    /// Windowed spending tracking (user, window_id)
    WindowSpending(Address, u64),
    /// Monthly spending tracking (user, month_id)
    MonthlySpending(Address, u64),
    /// Escalation configuration
    EscalationConfig,
    /// Pending large-spend approvals (spender, amount, timestamp)
    PendingApproval(Address),
}

/// Error codes for limit validation and enforcement.
pub mod ErrorCode {
    /// Invalid limit amount (negative, zero, or out of bounds)
    pub const INVALID_LIMIT: u32 = 0;
    /// Invalid limit amount (negative or zero)
    pub const INVALID_LIMIT_AMOUNT: u32 = 0;
    /// Invalid user address
    pub const INVALID_USER_ADDRESS: u32 = 1;
    /// Invalid reset window
    pub const INVALID_RESET_WINDOW: u32 = 2;
    /// Limit not found
    pub const LIMIT_NOT_FOUND: u32 = 3;
    /// Large spend requires admin approval
    pub const ESCALATION_APPROVAL_REQUIRED: u32 = 4;
    /// Pending approval not found or expired
    pub const APPROVAL_NOT_FOUND: u32 = 5;
}

/// Spending category for budget classification.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum BudgetCategory {
    Food,
    Transport,
    Rent,
    Entertainment,
}

/// Budget status.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum BudgetStatus {
    Active,
    Paused,
}

/// Budget record.
#[derive(Clone)]
#[contracttype]
pub struct Budget {
    pub owner: Address,
    pub limit: i128,
    pub spent: i128,
    pub status: BudgetStatus,
}

/// Events emitted by the spending limits contract.
pub struct LimitEvents;

impl LimitEvents {
    /// Event emitted when batch limit updates start.
    pub fn batch_started(env: &Env, batch_id: u64, request_count: u32) {
        let topics = (symbol_short!("batch"), symbol_short!("started"));
        env.events().publish(topics, (batch_id, request_count));
    }

    /// Event emitted when a limit is successfully updated.
    pub fn limit_updated(env: &Env, batch_id: u64, limit: &SpendingLimit) {
        let topics = (symbol_short!("limit"), symbol_short!("updated"), batch_id);
        env.events().publish(
            topics,
            (limit.user.clone(), limit.monthly_limit),
        );
    }

    /// Event emitted when a limit update fails.
    pub fn limit_update_failed(env: &Env, batch_id: u64, user: &Address, error_code: u32) {
        let topics = (symbol_short!("limit"), symbol_short!("failed"), batch_id);
        env.events().publish(topics, (user.clone(), error_code));
    }

    /// Event emitted when batch processing completes.
    pub fn batch_completed(
        env: &Env,
        batch_id: u64,
        successful: u32,
        failed: u32,
        total_value: i128,
    ) {
        let topics = (symbol_short!("batch"), symbol_short!("completed"), batch_id);
        env.events()
            .publish(topics, (successful, failed, total_value));
    }

    /// Event emitted for high-value limits.
    pub fn high_value_limit(env: &Env, batch_id: u64, user: &Address, amount: i128) {
        let topics = (symbol_short!("limit"), symbol_short!("highval"), batch_id);
        env.events().publish(topics, (user.clone(), amount));
    }

    /// Event emitted when a spending limit is exceeded.
    pub fn limit_exceeded(
        env: &Env,
        user: &Address,
        amount: i128,
        remaining_window: i128,
        remaining_monthly: i128,
    ) {
        let topics = (symbol_short!("limit"), symbol_short!("exceeded"));
        env.events()
            .publish(topics, (user.clone(), amount, remaining_window, remaining_monthly));
    }

    /// Event emitted when a large spend triggers escalation.
    pub fn escalation_triggered(env: &Env, user: &Address, amount: i128, level: u32) {
        let topics = (symbol_short!("escalation"), symbol_short!("triggered"));
        env.events().publish(topics, (user.clone(), amount, level));
    }

    /// Event emitted when an escalated spend is approved.
    pub fn escalation_approved(env: &Env, admin: &Address, user: &Address, amount: i128) {
        let topics = (symbol_short!("escalation"), symbol_short!("approved"));
        env.events()
            .publish(topics, (admin.clone(), user.clone(), amount));
    }

    /// Event emitted when escalation rules are configured.
    pub fn escalation_configured(
        env: &Env,
        small_threshold: i128,
        medium_threshold: i128,
        enabled: bool,
    ) {
        let topics = (symbol_short!("escalation"), symbol_short!("configured"));
        env.events()
            .publish(topics, (small_threshold, medium_threshold, enabled));
    }
}
