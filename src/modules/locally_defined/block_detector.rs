use rayon::prelude::*;

use crate::modules::{
    dist_algo_simulator::simulate_step, final_rule::FinalRule, validation_config::ValidationConfig,
};

pub fn detect_blocked_algorithms(
    algorithms: Vec<Vec<FinalRule>>,
    visibility: i16,
    max_steps: usize,
    initial_positions: &[Vec<(char, i16, i16)>],
) -> Vec<Vec<usize>> {
    algorithms
        .par_iter()
        .enumerate()
        .filter_map(|(i, algorithm)| {
            print!("Checking algorithm {}/{}...\r", i + 1, algorithms.len());
            let blocked_positions =
                detect_blocked_in_algo(algorithm, visibility, max_steps, initial_positions);
            Some(blocked_positions)
        })
        .collect()
}
pub fn detect_blocked_in_algo(
    algorithm: &[FinalRule],
    visibility: i16,
    max_steps: usize,
    initial_positions: &[Vec<(char, i16, i16)>],
) -> Vec<usize> {
    initial_positions
        .par_iter() // parallel iterator
        .enumerate()
        .filter_map(|(i, position)| {
            let validation_config = ValidationConfig::new(position.clone(), -10, 10, -10, 10);
            let mut robots_history: Vec<Vec<(char, i16, i16)>> = vec![position.clone()];

            for _ in 0..max_steps {
                if simulate_step(
                    &mut robots_history,
                    algorithm,
                    &validation_config,
                    visibility,
                ) {
                    return Some(i); // stop immediately when blocked
                }
            }
            None
        })
        .collect()
}
