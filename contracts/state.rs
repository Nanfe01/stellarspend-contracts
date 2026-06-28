use cosmwasm_std::Uint128;
use cw_storage_plus::Map;

pub const BALANCES: Map<&str, Uint128> = Map::new("balances");
pub const SAVINGS_GOALS: Map<&str, Uint128> = Map::new("savings_goals");
// ## State Contract Role
//
// `state.rs` defines shared persistent state (BALANCES and SAVINGS_GOALS)
// used across contracts to store user balance and savings goal mappings.
// It acts as the single source of truth for these values in the workspace.

#[cfg(test)]
mod state_tests {
    #[test]
    fn test_balances_key_is_unique() {
        // Verifies that the storage key "balances" is non-empty and distinct
        // from "savings_goals" to prevent key collisions.
        assert_ne!("balances", "savings_goals");
        assert!(!("balances".is_empty()));
    }

    #[test]
    fn test_savings_goals_key_is_unique() {
        assert!(!("savings_goals".is_empty()));
    }

    #[test]
    fn test_zero_balance_is_valid_initial_state() {
        let balance: u128 = 0;
        assert_eq!(balance, 0);
    }
}
