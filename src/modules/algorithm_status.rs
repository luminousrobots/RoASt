use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum AlgorithmStatus {
    Validated,
    ValidatedNotLd,
    Blocked,
    Cyclic,
    Timeout,
    Unknown,
}

impl AlgorithmStatus {
    pub fn label(&self) -> &'static str {
        match self {
            AlgorithmStatus::Validated => "[VALIDATED]",
            AlgorithmStatus::ValidatedNotLd => "[VALIDATED-NOT-LD]",
            AlgorithmStatus::Blocked => "[BLOCKED]",
            AlgorithmStatus::Cyclic => "[CYCLIC]",
            AlgorithmStatus::Timeout => "[TIMEOUT]",
            AlgorithmStatus::Unknown => "[UNKNOWN]",
        }
    }
    pub fn simple_label(&self) -> &'static str {
        match self {
            AlgorithmStatus::Validated => "validated",
            AlgorithmStatus::ValidatedNotLd => "validated_not_ld",
            AlgorithmStatus::Blocked => "blocked",
            AlgorithmStatus::Cyclic => "cyclic",
            AlgorithmStatus::Timeout => "timeout",
            AlgorithmStatus::Unknown => "unknown",
        }
    }
}
