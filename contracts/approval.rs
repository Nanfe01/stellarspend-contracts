use soroban_sdk::{contracttype, Address, Env};

use crate::storage::DataKey;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetApproval {
    pub id: u64,
    pub proposer: Address,
    pub approver: Address,
    pub budget_id: u64,
    pub proposed_amount: i128,
    pub approved: bool,
    pub created_at: u64,
    pub approved_at: Option<u64>,
}

pub fn save_approval(env: &Env, approval: &BudgetApproval) {
    env.storage()
        .persistent()
        .set(&DataKey::Approval(approval.id), approval);
}

pub fn get_approval(env: &Env, id: u64) -> Option<BudgetApproval> {
    env.storage()
        .persistent()
        .get(&DataKey::Approval(id))
}

pub fn update_approval(
    env: &Env,
    mut approval: BudgetApproval,
) -> BudgetApproval {
    approval.approved = true;
    approval.approved_at = Some(env.ledger().timestamp());

    save_approval(env, &approval);

    approval
}

pub fn delete_approval(env: &Env, id: u64) {
    env.storage()
        .persistent()
        .remove(&DataKey::Approval(id));
}