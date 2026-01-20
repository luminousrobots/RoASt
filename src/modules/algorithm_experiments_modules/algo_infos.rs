use serde::{Deserialize, Serialize};

use crate::modules::algorithm_experiments_modules::algo_info_by_robot_colors::AlgoInfoByRobotColors;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlgoInfos {
    pub total_activation: usize,
    pub by_robot_colors: Vec<AlgoInfoByRobotColors>,
}
