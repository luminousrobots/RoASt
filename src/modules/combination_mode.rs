#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum CombinationMode {
    Sequential,
    BiCombination,
    Parallel,
}
