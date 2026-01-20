#[derive(Debug)]
pub struct AlgorithmSnapshot {
    pub validated_ld: Vec<String>,
    pub validated_not_ld: Vec<String>,
    pub blocked: Vec<String>,
    pub cyclic: Vec<String>,
    pub timeout: Vec<String>,
}
