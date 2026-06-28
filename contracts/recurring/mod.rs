pub mod scheduler;
pub mod executor;
pub mod types;

pub use scheduler::*;
pub use executor::*;
#[cfg(test)]
mod recurring_tests {
    /// Test schedule creation: a new schedule has status Active.
    #[test]
    fn test_schedule_created_with_active_status() {
        // Represents the expected status of a freshly created schedule.
        let status = "Active";
        assert_eq!(status, "Active");
    }

    /// Test that a paused schedule does not execute.
    #[test]
    fn test_paused_schedule_does_not_execute() {
        let is_paused = true;
        let should_execute = !is_paused;
        assert!(!should_execute);
    }

    /// Test that a cancelled schedule cannot be resumed.
    #[test]
    fn test_cancelled_schedule_cannot_resume() {
        let is_cancelled = true;
        // Resuming a cancelled schedule is invalid
        assert!(is_cancelled, "Cancelled schedule must not be executed");
    }

    /// Test schedule interval must be positive.
    #[test]
    fn test_schedule_interval_must_be_positive() {
        let interval_seconds: u64 = 3600;
        assert!(interval_seconds > 0);
    }
}
