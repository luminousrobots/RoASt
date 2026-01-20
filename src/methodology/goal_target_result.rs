use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GoalTargetResult {
    pub execution_count: usize,
    pub result_path: String,
}
