use serde::Serialize;

#[derive(Serialize)]
pub struct AlgorithmData {
    pub name: String,
    pub status: String,
    pub experiments: Vec<String>,
}
