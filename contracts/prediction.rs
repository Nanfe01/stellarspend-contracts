#![no_std]
use soroban_sdk::{contractimpl, Env, Address, Vec, Map};

#[derive(Clone)]
pub struct Transaction {
    pub amount: i128,
    pub timestamp: u64,
}

#[derive(Clone)]
pub struct SpendingPrediction {
    pub projected_amount: i128,
    pub timestamp: u64,
}

#[contractimpl]
pub struct PredictionEngine;

#[contractimpl]
impl PredictionEngine {
    // Store a user's transaction history
    pub fn add_transaction(env: Env, user: Address, amount: i128, timestamp: u64) {
        let mut txs: Vec<Transaction> = env.storage().get(&user).unwrap_or_default();
        
        // Limit storage growth
        if txs.len() >= 50 {
            txs.remove(0);
        }

        txs.push(Transaction { amount, timestamp });
        env.storage().set(&user, &txs);
    }

    // Generate projected spending
    pub fn predict_spending(env: Env, user: Address) -> i128 {
        let txs: Vec<Transaction> = env.storage().get(&user).unwrap_or_default();
        let len = txs.len() as i128;
        if len == 0 {
            return 0;
        }

        // Simple rule: average last N transactions
        let sum: i128 = txs.iter().map(|t| t.amount).sum();
        let projected = sum / len;

        // Emit event
        env.events().publish(
            ("prediction_event", user.clone()),
            SpendingPrediction {
                projected_amount: projected,
                timestamp: env.ledger().timestamp(),
            },
        );

        projected
    }

    // Retrieve all transactions (for testing or analysis)
    pub fn get_transactions(env: Env, user: Address) -> Vec<Transaction> {
        env.storage().get(&user).unwrap_or_default()
    }
}
#[cfg(test)]
mod prediction_tests {
    use super::*;
    use soroban_sdk::{Address, Env};

    #[test]
    fn test_same_inputs_give_same_output() {
        let env = Env::default();
        let user = Address::generate(&env);
        PredictionEngine::add_transaction(env.clone(), user.clone(), 100, 1);
        PredictionEngine::add_transaction(env.clone(), user.clone(), 200, 2);
        let p1 = PredictionEngine::predict_spending(env.clone(), user.clone());
        let p2 = PredictionEngine::predict_spending(env.clone(), user.clone());
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_empty_user_returns_zero() {
        let env = Env::default();
        let user = Address::generate(&env);
        let result = PredictionEngine::predict_spending(env.clone(), user.clone());
        assert_eq!(result, 0);
    }

    #[test]
    fn test_single_transaction_predicts_itself() {
        let env = Env::default();
        let user = Address::generate(&env);
        PredictionEngine::add_transaction(env.clone(), user.clone(), 300, 1);
        let result = PredictionEngine::predict_spending(env.clone(), user.clone());
        assert_eq!(result, 300);
    }
}
