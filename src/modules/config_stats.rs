use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::modules::config_snapshot::ConfigSnapshot;

#[derive(Default)]
pub struct ConfigStats {
    validated_ld: Arc<AtomicUsize>,
    blocked_not_essential: Arc<AtomicUsize>,
    blocked: Arc<AtomicUsize>,
    cyclic: Arc<AtomicUsize>,
    timeout: Arc<AtomicUsize>,
}

impl ConfigStats {
    pub fn increment_validated(&self) {
        self.validated_ld.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_blocked_not_essential(&self) {
        self.blocked_not_essential.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_blocked(&self) {
        self.blocked.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_cyclic(&self) {
        self.cyclic.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_timeout(&self) {
        self.timeout.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> ConfigSnapshot {
        ConfigSnapshot {
            validated_ld: self.validated_ld.load(Ordering::Relaxed),
            blocked_not_essential: self.blocked_not_essential.load(Ordering::Relaxed),
            blocked: self.blocked.load(Ordering::Relaxed),
            cyclic: self.cyclic.load(Ordering::Relaxed),
            timeout: self.timeout.load(Ordering::Relaxed),
        }
    }
}
