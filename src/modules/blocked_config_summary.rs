#[derive(Debug, Clone)]
pub struct BlockedConfigSummary {
    pub algorithm_name: String,
    pub blocked_config_index: usize,
    pub blocked_initial_positions_index: usize,
    pub is_essential: bool, // NEW FIELD
}

impl BlockedConfigSummary {
    pub fn new(
        algorithm_name: String,
        blocked_config_index: usize,
        blocked_initial_positions_index: usize,
    ) -> Self {
        Self {
            algorithm_name,
            blocked_config_index,
            blocked_initial_positions_index,
            is_essential: true, // Default to true for backward compatibility
        }
    }

    pub fn new_with_flag(
        algorithm_name: String,
        blocked_config_index: usize,
        blocked_initial_positions_index: usize,
        is_essential: bool,
    ) -> Self {
        Self {
            algorithm_name,
            blocked_config_index,
            blocked_initial_positions_index,
            is_essential,
        }
    }
}
