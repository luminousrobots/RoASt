#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GenerationMode {
    All,
    ProgressiveValidationByLevels(usize),
}
