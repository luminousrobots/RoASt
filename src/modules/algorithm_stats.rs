use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

use crate::modules::{algorithm_snapshot::AlgorithmSnapshot, algorithm_status::AlgorithmStatus};

#[derive(Default)]
pub struct AlgorithmStats {
    validated_ld: Arc<Mutex<HashSet<String>>>,
    validated_not_ld: Arc<Mutex<HashSet<String>>>,
    blocked: Arc<Mutex<HashSet<String>>>,
    cyclic: Arc<Mutex<HashSet<String>>>,
    timeout: Arc<Mutex<HashSet<String>>>,
}

impl AlgorithmStats {
    pub fn insert(&self, status: AlgorithmStatus, algo_name: &str) {
        let target = match status {
            AlgorithmStatus::Validated => &self.validated_ld,
            AlgorithmStatus::ValidatedNotLd => &self.validated_not_ld,
            AlgorithmStatus::Blocked => &self.blocked,
            AlgorithmStatus::Cyclic => &self.cyclic,
            AlgorithmStatus::Timeout => &self.timeout,
            AlgorithmStatus::Unknown => return,
        };
        if let Ok(mut set) = target.lock() {
            set.insert(algo_name.to_string());
        }
    }

    pub fn snapshot(&self) -> AlgorithmSnapshot {
        AlgorithmSnapshot {
            validated_ld: Self::collect_sorted(&self.validated_ld),
            validated_not_ld: Self::collect_sorted(&self.validated_not_ld),
            blocked: Self::collect_sorted(&self.blocked),
            cyclic: Self::collect_sorted(&self.cyclic),
            timeout: Self::collect_sorted(&self.timeout),
        }
    }
    fn collect_sorted(set: &Arc<Mutex<HashSet<String>>>) -> Vec<String> {
        let mut items: Vec<String> = set
            .lock()
            .map(|guard| guard.iter().cloned().collect())
            .unwrap_or_default();
        items.sort();
        items
    }
}
