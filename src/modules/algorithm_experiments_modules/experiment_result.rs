use serde::{Deserialize, Serialize};

use crate::modules::{
    algorithm_experiments_modules::robot_criteria::RobotCriteria,
    execution_status::ExecutionStatus, grid_experiment::GridExperiment,
};

/// Result of running the algorithm on a single `GridExperiment`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResult {
    pub experiment_id: usize,
    pub grid_experiment: GridExperiment,
    pub status: ExecutionStatus,
    pub steps_taken: usize,
    pub cycle_len: usize,
    pub total_activation_in_cycle: usize,
    pub total_activation: usize,
    pub robots_metrics: Vec<RobotCriteria>,
}
