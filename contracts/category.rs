use soroban_sdk::{contracttype, String};

/// Categories that can be assigned to a savings goal.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoalCategory {
    Emergency,
    Education,
    Housing,
    Investment,
    Retirement,
    Travel,
    Vehicle,
    Business,
    Other(String),
}

impl GoalCategory {
    /// Returns true if the category is valid.
    pub fn is_valid(&self) -> bool {
        match self {
            GoalCategory::Other(name) => !name.is_empty(),
            _ => true,
        }
    }

    /// Returns the category as a string.
    pub fn as_str(&self) -> String {
        match self {
            GoalCategory::Emergency => String::from_str("Emergency"),
            GoalCategory::Education => String::from_str("Education"),
            GoalCategory::Housing => String::from_str("Housing"),
            GoalCategory::Investment => String::from_str("Investment"),
            GoalCategory::Retirement => String::from_str("Retirement"),
            GoalCategory::Travel => String::from_str("Travel"),
            GoalCategory::Vehicle => String::from_str("Vehicle"),
            GoalCategory::Business => String::from_str("Business"),
            GoalCategory::Other(name) => name.clone(),
        }
    }
}