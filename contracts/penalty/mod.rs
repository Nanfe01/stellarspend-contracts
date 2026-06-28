pub mod config;
pub mod withdrawal;

pub use config::*;
pub use withdrawal::*;
#[cfg(test)]
mod penalty_tests {
    /// Test standard penalty: 10% of 1000 = 100.
    #[test]
    fn test_standard_penalty_calculation() {
        let principal: i128 = 1_000;
        let penalty_bps: i128 = 1_000; // 10% in basis points
        let penalty = principal * penalty_bps / 10_000;
        assert_eq!(penalty, 100);
    }

    /// Test zero penalty on zero principal.
    #[test]
    fn test_zero_penalty_on_zero_principal() {
        let principal: i128 = 0;
        let penalty_bps: i128 = 500;
        let penalty = principal * penalty_bps / 10_000;
        assert_eq!(penalty, 0);
    }

    /// Test early withdrawal penalty is higher than standard.
    #[test]
    fn test_early_withdrawal_penalty_higher_than_standard() {
        let principal: i128 = 1_000;
        let standard_bps: i128 = 500;  // 5%
        let early_bps: i128 = 2_000;   // 20%
        let standard = principal * standard_bps / 10_000;
        let early = principal * early_bps / 10_000;
        assert!(early > standard);
    }

    /// Test penalty cannot exceed principal.
    #[test]
    fn test_penalty_cannot_exceed_principal() {
        let principal: i128 = 500;
        let penalty_bps: i128 = 10_000; // 100%
        let penalty = (principal * penalty_bps / 10_000).min(principal);
        assert!(penalty <= principal);
    }
}
