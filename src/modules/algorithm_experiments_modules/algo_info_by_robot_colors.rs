use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlgoInfoByRobotColors {
    pub robot_color: char,
    pub total_activation: usize,
    pub color_activations: usize,
    pub movement_activations: usize,
    pub rules_count: usize,
    pub idle_rules_count: usize,
    pub opacity_rule_count: usize,
}
