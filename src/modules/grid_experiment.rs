use serde::{Deserialize, Serialize};

use crate::modules::{grid_config::GridConfig, init_config::InitConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridExperiment {
    pub id: usize,
    pub grid_config: GridConfig,
    pub init_config: InitConfig,
}
