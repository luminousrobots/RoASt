use crate::modules::{grid_config::GridConfig, grid_experiment::GridExperiment};

use super::validation_config::ValidationConfig;

pub struct ExplorationHistory {
    matrix: Vec<Vec<bool>>,
}

impl ExplorationHistory {
    pub fn new(config: &GridConfig) -> Self {
        let full_width = (config.max_x - config.min_x + 1) as usize;
        let full_height = (config.max_y - config.min_y + 1) as usize;

        // Final matrix will exclude the outer border → so -2 from size
        let inner_width = full_width - 2;
        let inner_height = full_height - 2;

        // Matrix: initially all false (not visited)
        let matrix = vec![vec![false; inner_width]; inner_height];
        Self { matrix }
    }

    /// Set positions in the matrix based on the robot positions.
    pub fn set_positions(&mut self, positions: &[(char, i16, i16)], config: &GridConfig) {
        let inner_height = self.matrix.len();
        let inner_width = self.matrix[0].len();

        for &(_ch, x, y) in positions {
            // Only mark inner cells (not borders)
            if x > config.min_x && x < config.max_x && y > config.min_y && y < config.max_y {
                let inner_x = (x - config.min_x - 1) as usize;
                let inner_y = (y - config.min_y - 1) as usize;
                let flipped_y = inner_height - 1 - inner_y;

                if flipped_y < inner_height && inner_x < inner_width {
                    self.matrix[flipped_y][inner_x] = true;
                }
            }
        }
    }

    /// Print the exploration history matrix.
    pub fn print(&self) {
        for row in &self.matrix {
            println!("{:?}", row);
        }
    }
    pub fn is_fully_explored(&self) -> bool {
        // Check if all cells in all rows are `true`
        self.matrix.iter().all(|row| row.iter().all(|&cell| cell))
    }
}
