use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FamilyGroup {
    pub hash: String,
    pub signature: String,
    pub algorithm_count: usize,
    pub algorithms: Vec<super::algorithm_info::AlgorithmInfo>,
}
