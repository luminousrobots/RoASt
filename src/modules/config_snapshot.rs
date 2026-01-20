#[derive(Debug, Clone, Copy, Default)]
pub struct ConfigSnapshot {
    pub validated_ld: usize,
    pub blocked_not_essential: usize,
    pub blocked: usize,
    pub cyclic: usize,
    pub timeout: usize,
}
