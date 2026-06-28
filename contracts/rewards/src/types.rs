//! Data types and storage keys for the rewards contract.

use soroban_sdk::{contracttype, Address};

// ── Constants ─────────────────────────────────────────────────────────────────

/// TTL bump for persistent storage entries (~1 year in ledgers at ~5s/ledger).
pub const PERSISTENT_TTL_BUMP: u32 = 6_307_200;

// ── Storage keys ──────────────────────────────────────────────────────────────

/// All storage keys used by the rewards contract.
///
/// | Key | Storage tier | Description |
/// |---|---|---|
/// | `Admin` | Instance | Contract administrator address |
/// | `Initialized` | Instance | Initialization sentinel |
/// | `RewardBalance(Address)` | Persistent | Current claimable reward balance (stroops) |
/// | `LifetimeEarned(Address)` | Persistent | Total rewards ever earned (stroops) |
/// | `LifetimeClaimed(Address)` | Persistent | Total rewards ever claimed (stroops) |
/// | `RewardAccount(Address)` | Persistent | Full reward account metadata struct |
/// | `RewardTransaction(u64)` | Persistent | Individual reward transaction record by ID |
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Contract administrator address (instance storage).
    Admin,
    /// Initialization sentinel (instance storage).
    Initialized,
    /// Current claimable reward balance for an account (persistent storage).
    RewardBalance(Address),
    /// Cumulative rewards earned over the account lifetime (persistent storage).
    LifetimeEarned(Address),
    /// Cumulative rewards claimed over the account lifetime (persistent storage).
    LifetimeClaimed(Address),
    /// Full reward account metadata (persistent storage).
    RewardAccount(Address),
    /// Individual reward transaction record, keyed by transaction ID (persistent storage).
    RewardTransaction(u64),
}

// ── Structs ───────────────────────────────────────────────────────────────────

/// Metadata associated with a reward account.
///
/// Persisted under `DataKey::RewardAccount(address)`.
#[contracttype]
#[derive(Clone, Debug)]
pub struct RewardAccount {
    /// The owner of this reward account.
    pub owner: Address,
    /// Current claimable balance in stroops.
    pub balance: i128,
    /// Total rewards earned over the lifetime of the account in stroops.
    pub lifetime_earned: i128,
    /// Total rewards claimed over the lifetime of the account in stroops.
    pub lifetime_claimed: i128,
    /// Ledger sequence at which the account was first created.
    pub created_at: u64,
    /// Ledger sequence of the most recent balance update.
    pub last_updated: u64,
}
