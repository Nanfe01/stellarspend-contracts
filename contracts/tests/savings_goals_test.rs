use soroban_sdk::{testutils::Address as _, Address, Env, Symbol, Vec};

use savings::{SavingsContract, SavingsContractClient};
use savings_goals::{
    SavingsGoal, SavingsGoalProgress, SavingsGoalRequest, SavingsGoalsContract,
    SavingsGoalsContractClient,
};

struct TestContext {
    env: Env,
    admin: Address,
    savings_contract_id: Address,
    savings_client: SavingsContractClient<'static>,
    goals_client: SavingsGoalsContractClient<'static>,
}

fn setup_test() -> TestContext {
    let env = Env::default();
    env.mock_all_auths();

    let savings_contract_id = env.register(SavingsContract, ());
    let savings_client = SavingsContractClient::new(&env, &savings_contract_id);

    let goals_contract_id = env.register(SavingsGoalsContract, ());
    let goals_client = SavingsGoalsContractClient::new(&env, &goals_contract_id);

    let admin = Address::generate(&env);
    goals_client.initialize(&admin);

    TestContext {
        env,
        admin,
        savings_contract_id,
        savings_client,
        goals_client,
    }
}

fn create_goal_request(
    env: &Env,
    user: &Address,
    name: &str,
    target: i128,
    initial: i128,
) -> SavingsGoalRequest {
    let current_ledger = env.ledger().sequence() as u64;
    SavingsGoalRequest {
        user: user.clone(),
        goal_name: Symbol::new(env, name),
        target_amount: target,
        deadline: current_ledger + 1000,
        initial_contribution: initial,
        priority: 1,
        lock_duration_seconds: 0,
        penalty_bps: 0,
        expiration_seconds: 0,
    }
}

fn write_goal_to_savings_storage(
    env: &Env,
    contract_id: &Address,
    goal_id: u64,
    target: i128,
    saved: i128,
    completed: bool,
) {
    env.as_contract(contract_id, || {
        let key = savings::storage::DataKey::Goal(goal_id);
        let goal = savings::types::SavingsGoal {
            id: goal_id,
            target_amount: target,
            saved_amount: saved,
            completed,
        };
        env.storage().instance().set(&key, &goal);
    });
}

#[test]
fn test_goal_creation_contribution_and_milestones() {
    let ctx = setup_test();
    let user = Address::generate(&ctx.env);
    let target = 100_000_000i128;
    let reward_amount = 5_000_000i128;

    ctx.savings_client.set_reward_amount(&reward_amount);

    // Create a goal with 25% initial contribution (auto-triggers 25% milestone)
    let mut requests: Vec<SavingsGoalRequest> = Vec::new(&ctx.env);
    requests.push_back(create_goal_request(
        &ctx.env,
        &user,
        "emergency_fund",
        target,
        target * 25 / 100,
    ));
    let batch_result = ctx.goals_client.batch_set_savings_goals(&ctx.admin, &requests);
    assert_eq!(batch_result.successful, 1);

    // Verify milestone at 25% was auto-triggered by initial contribution
    let triggered = ctx.goals_client.get_triggered_milestone_percents(&1);
    assert!(triggered.contains(&25), "25% milestone should be triggered");

    // Contribute to reach 50%
    ctx.goals_client.contribute_to_goal(&user, &1, &(target * 25 / 100));
    let triggered = ctx.goals_client.get_triggered_milestone_percents(&1);
    assert!(triggered.contains(&50), "50% milestone should be triggered");
    assert_eq!(triggered.len(), 2);

    // Contribute to reach 75%
    ctx.goals_client.contribute_to_goal(&user, &1, &(target * 25 / 100));
    let triggered = ctx.goals_client.get_triggered_milestone_percents(&1);
    assert!(triggered.contains(&75), "75% milestone should be triggered");
    assert_eq!(triggered.len(), 3);

    // Contribute to reach 100%
    ctx.goals_client.contribute_to_goal(&user, &1, &(target * 25 / 100));
    let triggered = ctx.goals_client.get_triggered_milestone_percents(&1);
    assert!(triggered.contains(&100), "100% milestone should be triggered");
    assert_eq!(triggered.len(), 4);

    // Verify goal is complete
    let goal: SavingsGoal = ctx.goals_client.get_goal(&1).unwrap();
    assert!(goal.is_complete, "Goal should be marked complete");
    assert_eq!(goal.current_amount, target);

    // Verify goal progress
    let progress: SavingsGoalProgress = ctx.goals_client.get_goal_progress(&1).unwrap();
    assert!(progress.is_complete);
    assert_eq!(progress.progress_percentage, 100);

    // Verify goal was auto-closed
    let closed_at = ctx.goals_client.get_goal_closed_at(&1);
    assert!(closed_at.is_some(), "Goal should be closed");

    // Now test reward claiming via savings contract
    write_goal_to_savings_storage(
        &ctx.env,
        &ctx.savings_contract_id,
        1,
        target,
        target,
        true,
    );

    let claimed = ctx.savings_client.claim_reward(&user, &1);
    assert_eq!(claimed, reward_amount, "Should claim the set reward amount");
}

#[test]
fn test_double_claim_prevented() {
    let ctx = setup_test();
    let user = Address::generate(&ctx.env);
    let target = 100_000_000i128;
    let reward_amount = 5_000_000i128;

    ctx.savings_client.set_reward_amount(&reward_amount);

    write_goal_to_savings_storage(&ctx.env, &ctx.savings_contract_id, 1, target, target, true);

    let claimed = ctx.savings_client.claim_reward(&user, &1);
    assert_eq!(claimed, reward_amount);

    // Second claim should fail (double-claim prevention)
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        ctx.savings_client.claim_reward(&user, &1);
    }));
    assert!(result.is_err(), "Double claim should be rejected");
}

#[test]
fn test_incomplete_goal_no_reward() {
    let ctx = setup_test();
    let user = Address::generate(&ctx.env);
    let target = 100_000_000i128;

    ctx.savings_client.set_reward_amount(&5_000_000i128);

    write_goal_to_savings_storage(
        &ctx.env,
        &ctx.savings_contract_id,
        1,
        target,
        target / 2,
        false,
    );

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        ctx.savings_client.claim_reward(&user, &1);
    }));
    assert!(result.is_err(), "Claim for incomplete goal should be rejected");
}

#[test]
fn test_milestone_incremental_path() {
    let ctx = setup_test();
    let user = Address::generate(&ctx.env);
    let target = 200_000_000i128;

    // Create a goal with no initial contribution
    let mut requests: Vec<SavingsGoalRequest> = Vec::new(&ctx.env);
    requests.push_back(create_goal_request(&ctx.env, &user, "incremental", target, 0));
    let batch_result = ctx.goals_client.batch_set_savings_goals(&ctx.admin, &requests);
    assert_eq!(batch_result.successful, 1);

    // No milestones should be triggered yet
    let triggered = ctx.goals_client.get_triggered_milestone_percents(&1);
    assert_eq!(triggered.len(), 0);

    // Contribute 25%
    ctx.goals_client.contribute_to_goal(&user, &1, &(target * 25 / 100));
    let triggered = ctx.goals_client.get_triggered_milestone_percents(&1);
    assert!(triggered.contains(&25));
    assert_eq!(triggered.len(), 1);

    // Contribute another 25%
    ctx.goals_client.contribute_to_goal(&user, &1, &(target * 25 / 100));
    let triggered = ctx.goals_client.get_triggered_milestone_percents(&1);
    assert!(triggered.contains(&50));
    assert_eq!(triggered.len(), 2);

    // Contribute another 25%
    ctx.goals_client.contribute_to_goal(&user, &1, &(target * 25 / 100));
    let triggered = ctx.goals_client.get_triggered_milestone_percents(&1);
    assert!(triggered.contains(&75));
    assert_eq!(triggered.len(), 3);

    // Contribute final 25%
    ctx.goals_client.contribute_to_goal(&user, &1, &(target * 25 / 100));
    let triggered = ctx.goals_client.get_triggered_milestone_percents(&1);
    assert!(triggered.contains(&100));
    assert_eq!(triggered.len(), 4);

    // Verify goal is complete and auto-closed
    let goal = ctx.goals_client.get_goal(&1).unwrap();
    assert!(goal.is_complete);
    assert!(!goal.is_active, "Goal should be inactive after auto-close");
    assert!(ctx.goals_client.get_goal_closed_at(&1).is_some());
}
