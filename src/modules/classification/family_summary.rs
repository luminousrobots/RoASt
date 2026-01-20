use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FamilySummary {
    pub id: usize,
    pub family_name: String,
    pub total_groups: usize,
    pub details: String,
}
