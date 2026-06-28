#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::{
    storage::{
        get_lifetime_claimed, get_lifetime_earned, get_reward_account, get_reward_balance,
        has_reward_account, set_lifetime_claimed, set_lifetime_earned, set_reward_account,
        set_reward_balance,
    },
    types::RewardAccount,
    RewardsContract, RewardsContractClient,
};

fn setup() -> (Env, Address, RewardsContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(RewardsContract, ());
    let client = RewardsContractClient::new(&env, &contract_id);
    (env, admin, client)
}

// ── Contract entry-point tests (from #875) ────────────────────────────────────

#[test]
fn test_initialize_sets_admin() {
    let (_env, admin, client) = setup();
    client.initialize(&admin);
    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_is_initialized_returns_true_after_init() {
    let (_env, admin, client) = setup();
    assert!(!client.is_initialized());
    client.initialize(&admin);
    assert!(client.is_initialized());
}

#[test]
#[should_panic]
fn test_double_initialize_panics() {
    let (_env, admin, client) = setup();
    client.initialize(&admin);
    client.initialize(&admin);
}

#[test]
#[should_panic]
fn test_get_admin_before_init_panics() {
    let (_env, _admin, client) = setup();
    client.get_admin();
}

// ── Storage helper tests (#876) ───────────────────────────────────────────────
//
// Storage helpers must be invoked from within a contract context.
// We use `env.as_contract(&contract_id, || { ... })` to satisfy that
// requirement without needing a dedicated accessor entry point on the contract.

#[test]
fn test_reward_balance_defaults_to_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let user = Address::generate(&env);
    let contract_id = env.register(RewardsContract, ());
    env.as_contract(&contract_id, || {
        assert_eq!(get_reward_balance(&env, &user), 0);
    });
}

#[test]
fn test_set_and_get_reward_balance() {
    let env = Env::default();
    env.mock_all_auths();
    let user = Address::generate(&env);
    let contract_id = env.register(RewardsContract, ());
    env.as_contract(&contract_id, || {
        set_reward_balance(&env, &user, 5_000_000);
        assert_eq!(get_reward_balance(&env, &user), 5_000_000);
    });
}

#[test]
fn test_reward_balance_overwrite() {
    let env = Env::default();
    env.mock_all_auths();
    let user = Address::generate(&env);
    let contract_id = env.register(RewardsContract, ());
    env.as_contract(&contract_id, || {
        set_reward_balance(&env, &user, 1_000);
        set_reward_balance(&env, &user, 9_999);
        assert_eq!(get_reward_balance(&env, &user), 9_999);
    });
}

#[test]
fn test_lifetime_earned_defaults_to_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let user = Address::generate(&env);
    let contract_id = env.register(RewardsContract, ());
    env.as_contract(&contract_id, || {
        assert_eq!(get_lifetime_earned(&env, &user), 0);
    });
}

#[test]
fn test_set_and_get_lifetime_earned() {
    let env = Env::default();
    env.mock_all_auths();
    let user = Address::generate(&env);
    let contract_id = env.register(RewardsContract, ());
    env.as_contract(&contract_id, || {
        set_lifetime_earned(&env, &user, 100_000_000);
        assert_eq!(get_lifetime_earned(&env, &user), 100_000_000);
    });
}

#[test]
fn test_lifetime_claimed_defaults_to_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let user = Address::generate(&env);
    let contract_id = env.register(RewardsContract, ());
    env.as_contract(&contract_id, || {
        assert_eq!(get_lifetime_claimed(&env, &user), 0);
    });
}

#[test]
fn test_set_and_get_lifetime_claimed() {
    let env = Env::default();
    env.mock_all_auths();
    let user = Address::generate(&env);
    let contract_id = env.register(RewardsContract, ());
    env.as_contract(&contract_id, || {
        set_lifetime_claimed(&env, &user, 50_000_000);
        assert_eq!(get_lifetime_claimed(&env, &user), 50_000_000);
    });
}

#[test]
fn test_has_reward_account_false_before_creation() {
    let env = Env::default();
    env.mock_all_auths();
    let user = Address::generate(&env);
    let contract_id = env.register(RewardsContract, ());
    env.as_contract(&contract_id, || {
        assert!(!has_reward_account(&env, &user));
    });
}

#[test]
fn test_set_and_get_reward_account() {
    let env = Env::default();
    env.mock_all_auths();
    let user = Address::generate(&env);
    let contract_id = env.register(RewardsContract, ());

    env.as_contract(&contract_id, || {
        let record = RewardAccount {
            owner: user.clone(),
            balance: 2_000_000,
            lifetime_earned: 10_000_000,
            lifetime_claimed: 8_000_000,
            created_at: 100,
            last_updated: 200,
        };

        set_reward_account(&env, &user, &record);
        assert!(has_reward_account(&env, &user));

        let fetched = get_reward_account(&env, &user).expect("account should exist");
        assert_eq!(fetched.owner, user);
        assert_eq!(fetched.balance, 2_000_000);
        assert_eq!(fetched.lifetime_earned, 10_000_000);
        assert_eq!(fetched.lifetime_claimed, 8_000_000);
        assert_eq!(fetched.created_at, 100);
        assert_eq!(fetched.last_updated, 200);
    });
}

#[test]
fn test_reward_account_returns_none_when_absent() {
    let env = Env::default();
    env.mock_all_auths();
    let user = Address::generate(&env);
    let contract_id = env.register(RewardsContract, ());
    env.as_contract(&contract_id, || {
        assert!(get_reward_account(&env, &user).is_none());
    });
}

#[test]
fn test_balances_are_independent_per_user() {
    let env = Env::default();
    env.mock_all_auths();
    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);
    let contract_id = env.register(RewardsContract, ());
    env.as_contract(&contract_id, || {
        set_reward_balance(&env, &user_a, 1_000);
        set_reward_balance(&env, &user_b, 2_000);
        assert_eq!(get_reward_balance(&env, &user_a), 1_000);
        assert_eq!(get_reward_balance(&env, &user_b), 2_000);
    });
}
