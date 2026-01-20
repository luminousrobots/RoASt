use serde::{Deserialize, Serialize};

use crate::modules::position::Position;

/// Metrics tracked for a single robot during a single experiment.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RobotCriteria {
    pub robot_id: usize,
    pub robot_position: Position,
    pub total_activation: usize,
    pub color_activations: usize,
    pub movement_activations: usize,
    pub rule_count: usize,
    pub idle_rule_count: usize,
    pub positions: Vec<Position>,
}
