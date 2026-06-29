use soroban_sdk::{contracttype, Address, Env, String};

use crate::storage::DataKey;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoalTemplate {
    pub id: u64,
    pub owner: Address,
    pub name: String,
    pub target_amount: i128,
    pub interval: u64,
    pub duration: u64,
}

pub fn save_template(env: &Env, template: &GoalTemplate) {
    env.storage()
        .persistent()
        .set(&DataKey::Template(template.id), template);
}

pub fn get_template(env: &Env, id: u64) -> Option<GoalTemplate> {
    env.storage()
        .persistent()
        .get(&DataKey::Template(id))
}

pub fn delete_template(env: &Env, id: u64) {
    env.storage()
        .persistent()
        .remove(&DataKey::Template(id));
}

pub fn clone_template(
    env: &Env,
    original: &GoalTemplate,
    new_id: u64,
    new_owner: Address,
) -> GoalTemplate {
    let cloned = GoalTemplate {
        id: new_id,
        owner: new_owner,
        name: original.name.clone(),
        target_amount: original.target_amount,
        interval: original.interval,
        duration: original.duration,
    };

    save_template(env, &cloned);

    cloned
}