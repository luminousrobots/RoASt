use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ClassificationResult {
    pub total_algorithms: usize,
    pub classification_date: String,
    pub families: Vec<super::family_category::FamilyCategory>,
    pub summaries: Vec<super::family_summary::FamilySummary>,
}
