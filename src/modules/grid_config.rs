use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridConfig {
    // --- Grid configuration ---
    pub columns: i16,
    pub rows: i16,
    pub min_x: i16,
    pub max_x: i16,
    pub min_y: i16,
    pub max_y: i16,
    pub obstacle_position: (i16, i16),
}
