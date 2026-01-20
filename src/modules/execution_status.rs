use serde::{Deserialize, Serialize};

/// Represents the outcome of running an algorithm on a specific configuration
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum ExecutionStatus {
    /// Algorithm completed successfully with full exploration
    Validated,
    /// Algorithm works but lacks local determinism (some non-essential configs failed)
    BlockedNotEssential,
    /// Algorithm got blocked and couldn't proceed
    Blocked,
    /// Algorithm entered an infinite loop without completing
    Cycle,
    /// Algorithm exceeded maximum allowed steps
    Timeout,
}

impl ExecutionStatus {
    /// Returns a human-readable string representation
    pub fn to_string(&self) -> &'static str {
        match self {
            ExecutionStatus::Validated => "[VALIDATED]",
            ExecutionStatus::BlockedNotEssential => "[BLOCKED-NOT-ESSENTIAL]",
            ExecutionStatus::Blocked => "[BLOCKED]",
            ExecutionStatus::Cycle => "[CYCLIC]",
            ExecutionStatus::Timeout => "[TIMEOUT]",
        }
    }

    /// Returns a description of what this status means
    pub fn description(&self) -> &'static str {
        match self {
            ExecutionStatus::Validated => "Fully explored successfully",
            ExecutionStatus::BlockedNotEssential => "Non-essential config blocked - not LD",
            ExecutionStatus::Blocked => "Algorithm got blocked and cannot explore",
            ExecutionStatus::Cycle => "Exploration found cycle but not fully explored",
            ExecutionStatus::Timeout => "Exploration timeout - possible infinite loop",
        }
    }
}
