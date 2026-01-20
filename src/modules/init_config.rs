use serde::{Deserialize, Serialize};

use crate::modules::position::Position;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitConfig {
    pub current_position_index: usize,
    pub initial_positions: Vec<Position>,
    pub is_essential: bool,
}
