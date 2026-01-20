use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AlgorithmInfo {
    pub name: String,
    pub properties: Vec<(String, String)>,
}
