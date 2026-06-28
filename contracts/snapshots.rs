#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Address,
    Env,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SnapshotError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    InvalidPeriodDuration = 4,
    InvalidAmount = 5,
    InvalidTimestamp = 6,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetSnapshot {
    pub user: Address,
    pub amount: i128,
    pub period_start: u64,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    PeriodDuration,
    Snapshot(Address, u64),
}

#[contract]
pub struct SnapshotsContract;

#[contractimpl]
impl SnapshotsContract {
    pub fn initialize(env: Env, admin: Address, period_duration_seconds: u64) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, SnapshotError::AlreadyInitialized);
        }
        admin.require_auth();
        if period_duration_seconds == 0 {
            panic_with_error!(&env, SnapshotError::InvalidPeriodDuration);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::PeriodDuration, &period_duration_seconds);
        env.events().publish(
            (symbol_short!("snapshot"), symbol_short!("init")),
            (admin, period_duration_seconds),
        );
    }

    pub fn create_snapshot(
        env: Env,
        caller: Address,
        user: Address,
        amount: i128,
        period_start: u64,
    ) {
        caller.require_auth();
        Self::require_admin(&env, &caller);
        let period_duration: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PeriodDuration)
            .unwrap_or_else(|| panic_with_error!(&env, SnapshotError::NotInitialized));
        if period_start % period_duration != 0 {
            panic_with_error!(&env, SnapshotError::InvalidTimestamp);
        }
        let now = env.ledger().timestamp();
        if period_start > now {
            panic_with_error!(&env, SnapshotError::InvalidTimestamp);
        }
        let snapshot = BudgetSnapshot {
            user: user.clone(),
            amount,
            period_start,
            created_at: now,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Snapshot(user.clone(), period_start), &snapshot);
        env.events().publish(
            (symbol_short!("snapshot"), symbol_short!("created")),
            (user, period_start, amount, now),
        );
    }

    pub fn get_snapshot(env: Env, user: Address, timestamp: u64) -> Option<BudgetSnapshot> {
        let period_duration: u64 = env.storage().instance().get(&DataKey::PeriodDuration)?;
        if period_duration == 0 {
            return None;
        }
        let period_start = timestamp / period_duration * period_duration;
        env.storage()
            .persistent()
            .get(&DataKey::Snapshot(user, period_start))
    }

    pub fn get_snapshot_at_period(
        env: Env,
        user: Address,
        period_start: u64,
    ) -> Option<BudgetSnapshot> {
        env.storage()
            .persistent()
            .get(&DataKey::Snapshot(user, period_start))
    }

    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    pub fn get_period_duration(env: Env) -> Option<u64> {
        env.storage().instance().get(&DataKey::PeriodDuration)
    }

    fn require_admin(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(env, SnapshotError::NotInitialized));
        if caller != &admin {
            panic_with_error!(env, SnapshotError::Unauthorized);
        }
    }
}

#[cfg(test)]
mod snapshot_extra_tests {
    use super::*;
    use soroban_sdk::{Address, Env};

    fn setup(env: &Env) -> (soroban_sdk::Address, soroban_sdk::Address) {
        env.mock_all_auths();
        env.ledger().set_timestamp(0);
        let contract_id = env.register(SnapshotsContract, ());
        let admin = Address::generate(env);
        let client = SnapshotsContractClient::new(env, &contract_id);
        client.initialize(&admin, &86_400u64);
        (contract_id, admin)
    }

    #[test]
    fn test_create_and_restore_snapshot() {
        let env = Env::default();
        let (contract_id, admin) = setup(&env);
        env.ledger().set_timestamp(86_400);
        let client = SnapshotsContractClient::new(&env, &contract_id);
        let user = Address::generate(&env);
        client.create_snapshot(&admin, &user, &500i128, &86_400u64);
        let snap = client.get_snapshot_at_period(&user, &86_400u64);
        assert!(snap.is_some());
        assert_eq!(snap.unwrap().amount, 500);
    }

    #[test]
    fn test_snapshot_returns_none_for_missing() {
        let env = Env::default();
        let (contract_id, _) = setup(&env);
        let client = SnapshotsContractClient::new(&env, &contract_id);
        let user = Address::generate(&env);
        let snap = client.get_snapshot_at_period(&user, &86_400u64);
        assert!(snap.is_none());
    }

    #[test]
    fn test_period_duration_stored() {
        let env = Env::default();
        let (contract_id, _) = setup(&env);
        let client = SnapshotsContractClient::new(&env, &contract_id);
        let dur = client.get_period_duration();
        assert_eq!(dur, Some(86_400u64));
    }
}
