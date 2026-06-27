#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger as _},
    token::{StellarAssetClient, TokenClient},
    Address, Env,
};

use crate::{AllowancesContract, AllowancesContractClient};
use crate::types::{AllowanceError, Frequency};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Spin up the contract + a SAC token, mint `initial_balance` to `owner`,
/// and approve the contract to spend on behalf of `owner`.
fn setup(
    initial_balance: i128,
) -> (
    Env,
    AllowancesContractClient<'static>,
    Address, // owner
    Address, // recipient
    Address, // token
) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(AllowancesContract, ());
    let client = AllowancesContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    let owner = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Fund owner
    StellarAssetClient::new(&env, &token_id).mint(&owner, &initial_balance);

    // Approve contract to transfer on behalf of owner (large ledger approval)
    TokenClient::new(&env, &token_id).approve(
        &owner,
        &contract_id,
        &initial_balance,
        &(env.ledger().sequence() + 10_000),
    );

    (env, client, owner, recipient, token_id)
}

// ── Storage schema / contract deploy (#822) ───────────────────────────────────

#[test]
fn contract_deploys_and_count_starts_at_zero() {
    let (env, client, _, _, _) = setup(1_000);
    assert_eq!(client.allowance_count(), 0, "fresh contract must start at 0");
    let _ = env;
}

// ── create_allowance (#823) ────────────────────────────────────────────────────

#[test]
fn create_allowance_increments_count() {
    let (env, client, owner, recipient, token) = setup(1_000);
    let now = env.ledger().timestamp();

    let id = client.create_allowance(&owner, &recipient, &token, &100, &Frequency::Once, &now);
    assert_eq!(id, 1);
    assert_eq!(client.allowance_count(), 1);
}

#[test]
fn create_allowance_stores_correct_fields() {
    let (env, client, owner, recipient, token) = setup(1_000);
    let now = env.ledger().timestamp();

    let id = client.create_allowance(&owner, &recipient, &token, &250, &Frequency::Weekly, &now);
    let a = client.get_allowance(&id);

    assert_eq!(a.owner, owner);
    assert_eq!(a.recipient, recipient);
    assert_eq!(a.token, token);
    assert_eq!(a.amount, 250);
    assert!(matches!(a.frequency, Frequency::Weekly));
    assert_eq!(a.next_distribution, now);
    assert_eq!(a.distribution_count, 0);
    assert!(a.active);
}

#[test]
fn create_allowance_emits_created_event() {
    let (env, client, owner, recipient, token) = setup(1_000);
    let now = env.ledger().timestamp();

    let id = client.create_allowance(&owner, &recipient, &token, &100, &Frequency::Once, &now);

    let events = env.events().all();
    // At least one event should have been emitted
    assert!(!events.is_empty(), "expected at least one event");
    let _ = id;
}

#[test]
fn create_allowance_rejects_zero_amount() {
    let (env, client, owner, recipient, token) = setup(1_000);
    let now = env.ledger().timestamp();

    let err = client
        .try_create_allowance(&owner, &recipient, &token, &0, &Frequency::Once, &now)
        .err()
        .expect("must fail")
        .expect("contract error");
    assert_eq!(err, AllowanceError::InvalidAmount.into());
}

#[test]
fn create_allowance_rejects_negative_amount() {
    let (env, client, owner, recipient, token) = setup(1_000);
    let now = env.ledger().timestamp();

    let err = client
        .try_create_allowance(&owner, &recipient, &token, &-1, &Frequency::Once, &now)
        .err()
        .expect("must fail")
        .expect("contract error");
    assert_eq!(err, AllowanceError::InvalidAmount.into());
}

#[test]
fn owner_and_recipient_indices_are_populated() {
    let (env, client, owner, recipient, token) = setup(2_000);
    let now = env.ledger().timestamp();

    let id1 = client.create_allowance(&owner, &recipient, &token, &100, &Frequency::Once, &now);
    let id2 = client.create_allowance(&owner, &recipient, &token, &200, &Frequency::Weekly, &now);

    let owner_ids = client.get_owner_allowances(&owner);
    assert!(owner_ids.contains(&id1));
    assert!(owner_ids.contains(&id2));

    let recip_ids = client.get_recipient_allowances(&recipient);
    assert!(recip_ids.contains(&id1));
    assert!(recip_ids.contains(&id2));
}

// ── Weekly distribution (#824) ────────────────────────────────────────────────

#[test]
fn weekly_distribution_transfers_tokens() {
    let (env, client, owner, recipient, token) = setup(1_000);
    let now = env.ledger().timestamp();
    let token_client = TokenClient::new(&env, &token);

    let id = client.create_allowance(&owner, &recipient, &token, &300, &Frequency::Weekly, &now);

    // Move ledger past the first weekly window
    env.ledger().with_mut(|l| l.timestamp = now + 604_800 + 1);

    client.distribute(&id);

    assert_eq!(token_client.balance(&recipient), 300);
    assert_eq!(token_client.balance(&owner), 700);

    let a = client.get_allowance(&id);
    assert_eq!(a.distribution_count, 1);
    assert!(a.active, "weekly allowance must stay active after one distribution");
    // Next window should be ~2 weeks from start
    assert!(a.next_distribution > now + 604_800);
}

#[test]
fn weekly_interval_is_604800_seconds() {
    assert_eq!(Frequency::Weekly.interval_seconds(), Some(604_800));
}

#[test]
fn weekly_distribute_too_early_is_rejected() {
    let (env, client, owner, recipient, token) = setup(1_000);
    let now = env.ledger().timestamp();

    let id = client.create_allowance(&owner, &recipient, &token, &100, &Frequency::Weekly, &(now + 10_000));

    let err = client
        .try_distribute(&id)
        .err()
        .expect("must fail")
        .expect("contract error");
    assert_eq!(err, AllowanceError::TooEarlyToDistribute.into());
}

// ── Monthly distribution (#825) ───────────────────────────────────────────────

#[test]
fn monthly_distribution_transfers_tokens() {
    let (env, client, owner, recipient, token) = setup(1_000);
    let now = env.ledger().timestamp();
    let token_client = TokenClient::new(&env, &token);

    let id = client.create_allowance(&owner, &recipient, &token, &500, &Frequency::Monthly, &now);

    // Move ledger past the first monthly window
    env.ledger().with_mut(|l| l.timestamp = now + 2_592_000 + 1);

    client.distribute(&id);

    assert_eq!(token_client.balance(&recipient), 500);
    assert_eq!(token_client.balance(&owner), 500);

    let a = client.get_allowance(&id);
    assert_eq!(a.distribution_count, 1);
    assert!(a.active, "monthly allowance must stay active after one distribution");
    assert!(a.next_distribution > now + 2_592_000);
}

#[test]
fn monthly_interval_is_2592000_seconds() {
    assert_eq!(Frequency::Monthly.interval_seconds(), Some(2_592_000));
}

#[test]
fn monthly_distribute_too_early_is_rejected() {
    let (env, client, owner, recipient, token) = setup(1_000);
    let now = env.ledger().timestamp();

    // Start 30 days in the future
    let id = client.create_allowance(&owner, &recipient, &token, &100, &Frequency::Monthly, &(now + 2_592_000));

    let err = client
        .try_distribute(&id)
        .err()
        .expect("must fail")
        .expect("contract error");
    assert_eq!(err, AllowanceError::TooEarlyToDistribute.into());
}

// ── Once frequency ────────────────────────────────────────────────────────────

#[test]
fn once_allowance_deactivates_after_distribution() {
    let (env, client, owner, recipient, token) = setup(1_000);
    let now = env.ledger().timestamp();

    let id = client.create_allowance(&owner, &recipient, &token, &100, &Frequency::Once, &now);
    client.distribute(&id);

    let a = client.get_allowance(&id);
    assert!(!a.active, "Once allowance must be inactive after distribution");
}

#[test]
fn once_interval_is_none() {
    assert_eq!(Frequency::Once.interval_seconds(), None);
}

// ── Cancellation ──────────────────────────────────────────────────────────────

#[test]
fn cancel_allowance_deactivates_it() {
    let (env, client, owner, recipient, token) = setup(1_000);
    let now = env.ledger().timestamp();

    let id = client.create_allowance(&owner, &recipient, &token, &100, &Frequency::Weekly, &now);
    client.cancel_allowance(&id);

    let a = client.get_allowance(&id);
    assert!(!a.active);
}

#[test]
fn cancel_already_inactive_returns_error() {
    let (env, client, owner, recipient, token) = setup(1_000);
    let now = env.ledger().timestamp();

    let id = client.create_allowance(&owner, &recipient, &token, &100, &Frequency::Once, &now);
    client.distribute(&id); // deactivates it

    let err = client
        .try_cancel_allowance(&id)
        .err()
        .expect("must fail")
        .expect("contract error");
    assert_eq!(err, AllowanceError::AlreadyInactive.into());
}

#[test]
fn distribute_inactive_allowance_returns_error() {
    let (env, client, owner, recipient, token) = setup(1_000);
    let now = env.ledger().timestamp();

    let id = client.create_allowance(&owner, &recipient, &token, &100, &Frequency::Weekly, &now);
    client.cancel_allowance(&id);

    let err = client
        .try_distribute(&id)
        .err()
        .expect("must fail")
        .expect("contract error");
    assert_eq!(err, AllowanceError::AlreadyInactive.into());
}

#[test]
fn distribute_nonexistent_returns_error() {
    let (_env, client, ..) = setup(1_000);

    let err = client
        .try_distribute(&999)
        .err()
        .expect("must fail")
        .expect("contract error");
    assert_eq!(err, AllowanceError::NotFound.into());
}

// ── Approval workflow (#845) ────────────────────────────────────────────────

#[test]
fn without_config_large_allowance_is_active() {
    // No approval config → all allowances active on creation (backward compat).
    let (env, client, owner, recipient, token) = setup(10_000);
    let now = env.ledger().timestamp();

    let id = client.create_allowance(&owner, &recipient, &token, &9_000, &Frequency::Once, &now);
    let a = client.get_allowance(&id);
    assert!(a.active);
    assert!(!a.pending_approval);
}

#[test]
fn set_approval_config_rejects_non_positive_threshold() {
    let (env, client, _owner, _recipient, _token) = setup(1_000);
    let approver = Address::generate(&env);

    let err = client
        .try_set_approval_config(&approver, &0)
        .err()
        .expect("must fail")
        .expect("contract error");
    assert_eq!(err, AllowanceError::InvalidThreshold.into());
}

#[test]
fn over_threshold_allowance_is_pending_and_inactive() {
    let (env, client, owner, recipient, token) = setup(10_000);
    let now = env.ledger().timestamp();
    let approver = Address::generate(&env);

    client.set_approval_config(&approver, &100);

    let id = client.create_allowance(&owner, &recipient, &token, &500, &Frequency::Once, &now);
    let a = client.get_allowance(&id);
    assert!(a.pending_approval, "over-threshold allowance must be pending");
    assert!(!a.active, "unapproved allowance must remain inactive");
}

#[test]
fn at_or_below_threshold_allowance_is_active() {
    let (env, client, owner, recipient, token) = setup(10_000);
    let now = env.ledger().timestamp();
    let approver = Address::generate(&env);

    client.set_approval_config(&approver, &100);

    let id = client.create_allowance(&owner, &recipient, &token, &100, &Frequency::Once, &now);
    let a = client.get_allowance(&id);
    assert!(a.active);
    assert!(!a.pending_approval);
}

#[test]
fn unapproved_allowance_cannot_distribute() {
    let (env, client, owner, recipient, token) = setup(10_000);
    let now = env.ledger().timestamp();
    let approver = Address::generate(&env);

    client.set_approval_config(&approver, &100);
    let id = client.create_allowance(&owner, &recipient, &token, &500, &Frequency::Once, &now);

    let err = client
        .try_distribute(&id)
        .err()
        .expect("must fail")
        .expect("contract error");
    assert_eq!(err, AllowanceError::ApprovalRequired.into());
}

#[test]
fn approve_activates_and_allows_distribution() {
    let (env, client, owner, recipient, token) = setup(10_000);
    let now = env.ledger().timestamp();
    let approver = Address::generate(&env);

    client.set_approval_config(&approver, &100);
    let id = client.create_allowance(&owner, &recipient, &token, &500, &Frequency::Once, &now);

    client.approve_allowance(&id);
    let a = client.get_allowance(&id);
    assert!(a.active);
    assert!(!a.pending_approval);

    // Now distribution succeeds.
    client.distribute(&id);
    assert_eq!(client.get_allowance(&id).distribution_count, 1);
}

#[test]
fn approve_rejects_non_pending_allowance() {
    let (env, client, owner, recipient, token) = setup(10_000);
    let now = env.ledger().timestamp();
    let approver = Address::generate(&env);

    client.set_approval_config(&approver, &100);
    // Below threshold → active, not pending.
    let id = client.create_allowance(&owner, &recipient, &token, &50, &Frequency::Once, &now);

    let err = client
        .try_approve_allowance(&id)
        .err()
        .expect("must fail")
        .expect("contract error");
    assert_eq!(err, AllowanceError::NotPendingApproval.into());
}

#[test]
fn approve_without_config_fails() {
    let (env, client, _owner, _recipient, _token) = setup(1_000);
    let err = client
        .try_approve_allowance(&1)
        .err()
        .expect("must fail")
        .expect("contract error");
    assert_eq!(err, AllowanceError::ApproverNotConfigured.into());
}

// ── Ownership transfer (#845) ───────────────────────────────────────────────

#[test]
fn transfer_ownership_reassigns_owner_and_indices() {
    let (env, client, owner, recipient, token) = setup(2_000);
    let now = env.ledger().timestamp();
    let new_owner = Address::generate(&env);

    let id = client.create_allowance(&owner, &recipient, &token, &100, &Frequency::Once, &now);

    client.transfer_ownership(&id, &new_owner);

    let a = client.get_allowance(&id);
    assert_eq!(a.owner, new_owner, "new owner controls the allowance");

    assert!(client.get_owner_allowances(&new_owner).contains(&id));
    assert!(!client.get_owner_allowances(&owner).contains(&id));
}

#[test]
fn transfer_ownership_then_new_owner_can_cancel() {
    let (env, client, owner, recipient, token) = setup(2_000);
    let now = env.ledger().timestamp();
    let new_owner = Address::generate(&env);

    let id = client.create_allowance(&owner, &recipient, &token, &100, &Frequency::Once, &now);
    client.transfer_ownership(&id, &new_owner);

    // New owner-controlled action succeeds; allowance becomes inactive.
    client.cancel_allowance(&id);
    assert!(!client.get_allowance(&id).active);
}
