use soroban_sdk::{Env, Address};

use crate::penalty::config::{get_penalty_percent, get_treasury};

pub fn apply_penalty_withdrawal(
    env: &Env,
    token: &Address,
    contract: &Address,
    user: &Address,
    amount: i128,
) {
    let penalty_percent = get_penalty_percent(env);

    let penalty: i128 = amount * penalty_percent as i128 / 100;
    let withdrawable: i128 = amount - penalty;

    let treasury = get_treasury(env);

    // transfer to user
    token.transfer(contract, user, &withdrawable);

    // send penalty to treasury
    token.transfer(contract, &treasury, &penalty);
}