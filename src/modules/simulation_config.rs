pub type Target = (
    usize,
    Vec<(char, i16, i16)>,
    Vec<(i16, i16)>,
    Vec<(i16, i16)>,
);
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SimulationConfig {
    pub initial_positions: Vec<(char, i16, i16)>,
    pub targets: Vec<Target>,
    pub boundary: Option<(i16, i16, i16, i16)>,
    pub wall: (Option<i16>, Option<i16>),
}

impl SimulationConfig {
    pub fn new(
        initial_positions: Vec<(char, i16, i16)>,
        targets: Vec<Target>,
        boundary: Option<(i16, i16, i16, i16)>,
        wall: (Option<i16>, Option<i16>),
    ) -> Self {
        Self {
            initial_positions,
            targets,
            boundary,
            wall,
        }
    }
}
