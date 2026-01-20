use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FamilyCategory {
    pub family_number: usize,
    pub title: String,
    pub description: String,
    pub total_families: usize,
    pub groups: Vec<super::family_group::FamilyGroup>,
}
