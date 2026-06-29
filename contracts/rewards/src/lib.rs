//! # Rewards Contract
//!
//! A Soroban smart contract dedicated to reward management for StellarSpend.
//! Provides the foundation for incentivising responsible financial behaviour
//! within the protocol.

#![no_std]

pub mod rewards;
pub mod storage;
pub mod types;
pub mod validation;

use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Env};

use crate::rewards::{credit_reward, register_reward_account};
use crate::storage::get_reward_account;
pub use crate::types::{DataKey, RewardAccount, RewardStatus, RewardTransaction, RewardType};

/// Error codes for the rewards contract.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum RewardsError {
    /// Contract has not been initialised.
    NotInitialized = 1,
    /// Caller is not authorised to perform this action.
    Unauthorized = 2,
    /// Contract has already been initialised.
    AlreadyInitialized = 3,
    /// Reward account already exists for this address.
    AccountAlreadyRegistered = 4,
    /// Reward amount must be greater than zero.
    InvalidAmount = 5,
    /// No reward account found for the given address.
    AccountNotFound = 6,
    /// Arithmetic overflow would occur.
    Overflow = 7,
}

impl From<RewardsError> for soroban_sdk::Error {
    fn from(e: RewardsError) -> Self {
        soroban_sdk::Error::from_contract_error(e as u32)
    }
}

#[contract]
pub struct RewardsContract;

#[contractimpl]
impl RewardsContract {
    /// Initialises the contract with an admin address.
    ///
    /// # Arguments
    /// * `env`   - The Soroban environment.
    /// * `admin` - The address that will administer this contract.
    ///
    /// # Errors
    /// Panics with `AlreadyInitialized` if called more than once.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Initialized) {
            panic_with_error!(&env, RewardsError::AlreadyInitialized);
        }

        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Initialized, &true);

        env.events().publish(("rewards", "initialized"), admin);
    }

    /// Returns the current admin address.
    ///
    /// # Errors
    /// Panics with `NotInitialized` if the contract has not been initialised.
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(&env, RewardsError::NotInitialized))
    }

    /// Returns `true` if the contract has been initialised.
    pub fn is_initialized(env: Env) -> bool {
        env.storage().instance().has(&DataKey::Initialized)
    }

    /// Registers a new reward account for `participant`.
    ///
    /// The caller must be the participant themselves — they authorise their own
    /// registration. Default values (all zeros) are stored for balance,
    /// lifetime earned, and lifetime claimed.
    ///
    /// # Errors
    /// Panics with `NotInitialized` if the contract has not been initialised.
    /// Panics with `AccountAlreadyRegistered` if the account already exists.
    pub fn register_account(env: Env, participant: Address) {
        participant.require_auth();
        match register_reward_account(&env, &participant) {
            Ok(()) => {}
            Err(e) => panic_with_error!(&env, e),
        }
    }

    /// Returns the `RewardAccount` metadata for `participant`, if registered.
    pub fn get_account(env: Env, participant: Address) -> Option<RewardAccount> {
        get_reward_account(&env, &participant)
    }

    /// Credits `amount` reward points to `participant`'s account.
    ///
    /// Only the contract admin may call this entry point. The amount must be
    /// strictly positive. Both the claimable balance and the lifetime-earned
    /// total are updated atomically. A [`RewardTransaction`] record is
    /// persisted and a `reward_credited` event is emitted.
    ///
    /// # Errors
    /// Panics with `NotInitialized` if the contract has not been initialised.
    /// Panics with `Unauthorized` if the caller is not the admin.
    /// Panics with `AccountNotFound` if `participant` has no reward account.
    /// Panics with `InvalidAmount` if `amount` is zero or negative.
    /// Panics with `Overflow` if crediting would overflow `i128`.
    pub fn credit_reward(
        env: Env,
        participant: Address,
        amount: i128,
        reward_type: RewardType,
    ) -> RewardTransaction {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(&env, RewardsError::NotInitialized));
        admin.require_auth();

        match credit_reward(&env, &participant, amount, reward_type) {
            Ok(tx) => tx,
            Err(e) => panic_with_error!(&env, e),
        }
    }

    // ── Internal helpers ──────────────────────────────────────────────────

    /// Asserts that `caller` is the contract admin.
    #[allow(dead_code)]
    fn require_admin(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(env, RewardsError::NotInitialized));

        if *caller != admin {
            panic_with_error!(env, RewardsError::Unauthorized);
        }
    }
}

#[cfg(test)]
mod test;
