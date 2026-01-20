// ============================================================================
// RESULT ANALYSIS
// ============================================================================

use crate::modules::execution_status::ExecutionStatus;

/// Statistics about how an algorithm performed across all configurations
#[derive(Debug)]
pub struct AlgorithmMetrics {
    pub validated_ld: usize,     // Fully validated configurations
    pub validated_not_ld: usize, // Validated but not locally deterministic
    pub blocked: usize,          // Blocked configurations
    pub cyclic: usize,           // Cyclic configurations
    pub timeout: usize,          // Timed out configurations
}

impl AlgorithmMetrics {
    /// Count outcomes from configuration results
    pub fn from_outcomes(config_outcomes: &[ExecutionStatus]) -> Self {
        Self {
            validated_ld: count_status(config_outcomes, ExecutionStatus::Validated),
            validated_not_ld: count_status(config_outcomes, ExecutionStatus::BlockedNotEssential),
            blocked: count_status(config_outcomes, ExecutionStatus::Blocked),
            cyclic: count_status(config_outcomes, ExecutionStatus::Cycle),
            timeout: count_status(config_outcomes, ExecutionStatus::Timeout),
        }
    }
}

/// Counts how many configurations have a specific status
fn count_status(config_outcomes: &[ExecutionStatus], target_status: ExecutionStatus) -> usize {
    config_outcomes
        .iter()
        .filter(|status| **status == target_status)
        .count()
}
