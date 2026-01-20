use serde::{Deserialize, Serialize};

use super::full_rule::FullRule;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Algorithm {
    pub name: String,
    pub rules: Vec<FullRule>,
}
