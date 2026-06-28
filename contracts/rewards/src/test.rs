#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::{RewardsContract, RewardsContractClient};

fn setup() -> (Env, Address, RewardsContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(RewardsContract, ());
    let client = RewardsContractClient::new(&env, &contract_id);
    (env, admin, client)
}

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
