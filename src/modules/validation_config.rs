use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValidationConfig {
    pub initial_positions: Vec<(char, i16, i16)>, // initial robot states
    pub min_x: i16,
    pub max_x: i16,
    pub min_y: i16,
    pub max_y: i16,
}

impl ValidationConfig {
    pub fn new(
        initial_positions: Vec<(char, i16, i16)>,
        min_x: i16,
        max_x: i16,
        min_y: i16,
        max_y: i16,
    ) -> Self {
        Self {
            initial_positions,
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }
}
