//! # Rewards Contract
//!
//! A Soroban smart contract dedicated to reward management for StellarSpend.
//! Provides the foundation for incentivising responsible financial behaviour
//! within the protocol.

#![no_std]

pub mod storage;
pub mod types;

use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Env};

pub use crate::types::{DataKey, RewardAccount};

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
