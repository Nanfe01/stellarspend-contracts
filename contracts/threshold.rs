use soroban_sdk::{contracttype, Env};

use crate::storage::DataKey;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ThresholdAction {
    Notify,
    FreezeBudget,
    LockSpending,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetThreshold {
    pub id: u64,
    pub budget_id: u64,
    pub amount: i128,
    pub action: ThresholdAction,
    pub executed: bool,
}

pub fn save_threshold(env: &Env, threshold: &BudgetThreshold) {
    env.storage()
        .persistent()
        .set(&DataKey::Threshold(threshold.id), threshold);
}

pub fn get_threshold(
    env: &Env,
    id: u64,
) -> Option<BudgetThreshold> {
    env.storage()
        .persistent()
        .get(&DataKey::Threshold(id))
}

pub fn delete_threshold(env: &Env, id: u64) {
    env.storage()
        .persistent()
        .remove(&DataKey::Threshold(id));
}

pub fn mark_executed(
    env: &Env,
    mut threshold: BudgetThreshold,
) -> BudgetThreshold {
    threshold.executed = true;

    save_threshold(env, &threshold);

    threshold
}