use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::modules::{
    algorithm_experiments_modules::{
        algo_info_by_robot_colors::AlgoInfoByRobotColors, algo_infos::AlgoInfos,
        experiment_result::ExperimentResult,
    },
    algorithm_status::AlgorithmStatus,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmExperiments {
    pub name: String,
    pub status: AlgorithmStatus,
    pub infos: AlgoInfos,
    pub experiments: Vec<ExperimentResult>,
}

impl AlgorithmExperiments {
    pub fn hash_by_rules_count(&self) -> String {
        let total: usize = self
            .infos
            .by_robot_colors
            .iter()
            .map(|info| info.rules_count)
            .sum();
        total.to_string()
    }
    pub fn hash_by_idle_rules_count(&self) -> String {
        let total: usize = self
            .infos
            .by_robot_colors
            .iter()
            .map(|info| info.idle_rules_count)
            .sum();
        total.to_string()
    }

    pub fn hash_by_opac_rules_count(&self) -> String {
        let total: usize = self
            .infos
            .by_robot_colors
            .iter()
            .map(|info| info.opacity_rule_count)
            .sum();
        total.to_string()
    }
    // ...existing code...

    pub fn hash_family_by_rules_count_by_colors(&self) -> String {
        self.infos
            .by_robot_colors
            .iter()
            .map(|info| format!("{}:{}", info.robot_color, info.rules_count))
            .collect::<Vec<_>>()
            .join("-")
    }

    pub fn hash_family_by_idle_rules_count_by_colors(&self) -> String {
        self.infos
            .by_robot_colors
            .iter()
            .map(|info| format!("{}:{}", info.robot_color, info.idle_rules_count))
            .collect::<Vec<_>>()
            .join("-")
    }

    pub fn hash_family_by_opacity_rules_count_by_colors(&self) -> String {
        self.infos
            .by_robot_colors
            .iter()
            .map(|info| format!("{}:{}", info.robot_color, info.opacity_rule_count))
            .collect::<Vec<_>>()
            .join("-")
    }

    // ...existing code...

    pub fn hash_by_rules_count_in_executions(&self) -> String {
        self.experiments
            .iter()
            .map(|exp| {
                // Calculate total rules used in this experiment
                let total_rules: usize = exp
                    .robots_metrics
                    .iter()
                    .map(|robot| robot.rule_count)
                    .sum();
                // format!("{:04}", total_rules)
                total_rules.to_string()
            })
            .collect::<Vec<_>>()
            .join("-")
    }
    pub fn hash_by_total_activation_in_executions(&self) -> String {
        self.experiments
            .iter()
            .map(|exp| {
                // Calculate total rules used in this experiment
                let total_rules: usize = exp
                    .robots_metrics
                    .iter()
                    .map(|robot| robot.total_activation)
                    .sum();
                // format!("{:04}", total_rules)
                total_rules.to_string()
            })
            .collect::<Vec<_>>()
            .join("-")
    }
    pub fn hash_by_color_activation_in_executions(&self) -> String {
        self.experiments
            .iter()
            .map(|exp| {
                // Calculate total rules used in this experiment
                let total_rules: usize = exp
                    .robots_metrics
                    .iter()
                    .map(|robot| robot.color_activations)
                    .sum();
                // format!("{:04}", total_rules)
                total_rules.to_string()
            })
            .collect::<Vec<_>>()
            .join("-")
    }

    pub fn hash_by_movement_activation_in_executions(&self) -> String {
        self.experiments
            .iter()
            .map(|exp| {
                // Calculate total rules used in this experiment
                let total_rules: usize = exp
                    .robots_metrics
                    .iter()
                    .map(|robot| robot.movement_activations)
                    .sum();
                // format!("{:04}", total_rules)
                total_rules.to_string()
            })
            .collect::<Vec<_>>()
            .join("-")
    }

    pub fn hash_by_rules_count_in_executions_by_robot(&self) -> String {
        self.experiments
            .iter()
            .map(|exp| {
                // Calculate total rules used in this experiment
                exp.robots_metrics
                    .iter()
                    .map(|robot| format!("{}:{}", robot.robot_id, robot.rule_count))
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect::<Vec<_>>()
            .join("-")
    }
    pub fn hash_by_activation_in_executions_by_robot(&self) -> String {
        self.experiments
            .iter()
            .map(|exp| {
                // Create string like "r:14,g:20,b:15" for each experiment
                exp.robots_metrics
                    .iter()
                    .map(|robot| format!("{}:{}", robot.robot_id, robot.total_activation))
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect::<Vec<_>>()
            .join("-")
    }

    pub fn hash_by_color_activation_in_executions_by_robot(&self) -> String {
        self.experiments
            .iter()
            .map(|exp| {
                // Create string like "r:14,g:20,b:15" for each experiment
                exp.robots_metrics
                    .iter()
                    .map(|robot| format!("{}:{}", robot.robot_id, robot.color_activations))
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect::<Vec<_>>()
            .join("-")
    }
    pub fn hash_by_movement_activation_in_executions_by_robot(&self) -> String {
        self.experiments
            .iter()
            .map(|exp| {
                // Create string like "r:14,g:20,b:15" for each experiment
                exp.robots_metrics
                    .iter()
                    .map(|robot| format!("{}:{}", robot.robot_id, robot.movement_activations))
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect::<Vec<_>>()
            .join("-")
    }

    pub fn hash_by_steps_taken_in_executions(&self) -> String {
        self.experiments
            .iter()
            .map(|exp| exp.steps_taken.to_string())
            .collect::<Vec<_>>()
            .join("-")
    }

    pub fn hash_by_cycle_len_in_executions(&self) -> String {
        self.experiments
            .iter()
            .map(|exp| exp.cycle_len.to_string())
            .collect::<Vec<_>>()
            .join("-")
    }

    pub fn hash_by_total_activation_in_cycle_in_executions(&self) -> String {
        self.experiments
            .iter()
            .map(|exp| exp.total_activation_in_cycle.to_string())
            .collect::<Vec<_>>()
            .join("-")
    }

    pub fn hash_by_cycle_paths_in_executions(&self) -> String {
        self.experiments
            .iter()
            .map(|exp| {
                let cycle_len = exp.cycle_len;

                // Get positions from the last cycle_len+1 positions (cycle_len steps need cycle_len+1 states)
                exp.robots_metrics
                    .iter()
                    .map(|robot| {
                        let total_positions = robot.positions.len();

                        // Calculate start index for cycle positions
                        // cycle_len steps require cycle_len+1 positions
                        let start_idx = if cycle_len > 0 && total_positions > cycle_len {
                            total_positions - cycle_len - 1
                        } else {
                            0
                        };

                        let positions_str = robot
                            .positions
                            .iter()
                            .skip(start_idx)
                            .map(|pos| format!("({},{})", pos.1, pos.2))
                            .collect::<Vec<_>>()
                            .join(";");
                        format!("{}:[{}]", robot.robot_id, positions_str)
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect::<Vec<_>>()
            .join("-")
    }

    pub fn hash_by_paths_before_cycle_in_executions(&self) -> String {
        self.experiments
            .iter()
            .map(|exp| {
                let cycle_len = exp.cycle_len;

                // Get positions from the last cycle_len+1 positions (cycle_len steps need cycle_len+1 states)
                exp.robots_metrics
                    .iter()
                    .map(|robot| {
                        let total_positions = robot.positions.len();

                        // Calculate start index for cycle positions
                        // cycle_len steps require cycle_len+1 positions
                        let start_idx = if cycle_len > 0 && total_positions > cycle_len {
                            total_positions - cycle_len
                        } else {
                            0
                        };

                        let positions_str = robot
                            .positions
                            .iter()
                            .take(start_idx)
                            .map(|pos| format!("({},{})", pos.1, pos.2))
                            .collect::<Vec<_>>()
                            .join(";");
                        format!("{}:[{}]", robot.robot_id, positions_str)
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect::<Vec<_>>()
            .join("-")
    }

    pub fn hash_by_activation_in_executions_before_cycle(&self) -> String {
        self.experiments
            .iter()
            .map(|exp| {
                //do th sum oc total activation of robots
                let total_activation: usize = exp
                    .robots_metrics
                    .iter()
                    .map(|robot| robot.total_activation)
                    .sum();
                let total_activation_before_cycle =
                    total_activation - exp.total_activation_in_cycle;
                format!("{}", total_activation)
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    pub fn hash_by_color_in_executions(&self) -> String {
        self.experiments
            .iter()
            .map(|exp| {
                // Get full path of each robot with robot id
                exp.robots_metrics
                    .iter()
                    .map(|robot| {
                        let positions_str = robot
                            .positions
                            .iter()
                            .map(|pos| format!("({})", pos.0))
                            .collect::<Vec<_>>()
                            .join(";");
                        format!("{}:[{}]", robot.robot_id, positions_str)
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect::<Vec<_>>()
            .join("-")
    }
    pub fn hash_by_paths_in_executions(&self) -> String {
        self.experiments
            .iter()
            .map(|exp| {
                // Get full path of each robot with robot id
                exp.robots_metrics
                    .iter()
                    .map(|robot| {
                        let positions_str = robot
                            .positions
                            .iter()
                            .map(|pos| format!("({},{})", pos.1, pos.2))
                            .collect::<Vec<_>>()
                            .join(";");
                        format!("{}:[{}]", robot.robot_id, positions_str)
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect::<Vec<_>>()
            .join("-")
    }

    pub fn hash_by_positions_in_executions(&self) -> String {
        self.experiments
            .iter()
            .map(|exp| {
                // Get full path of each robot with robot id
                exp.robots_metrics
                    .iter()
                    .map(|robot| {
                        let positions_str = robot
                            .positions
                            .iter()
                            .map(|pos| format!("({},{},{})", pos.0, pos.1, pos.2))
                            .collect::<Vec<_>>()
                            .join(";");
                        format!("{}:[{}]", robot.robot_id, positions_str)
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Get comparison signatures for each experiment (positions with colors)
    pub fn get_experiment_signatures(&self) -> Vec<String> {
        self.experiments
            .iter()
            .map(|exp| {
                // Get initial positions from init_config
                let init_config = &exp.grid_experiment.init_config;
                let grid_config = &exp.grid_experiment.grid_config;

                // Calculate normalized positions (relative to obstacle at 0,0)
                let obstacle_x = grid_config.obstacle_position.0;
                let obstacle_y = grid_config.obstacle_position.1;

                let mut normalized_positions: Vec<(char, i16, i16)> = init_config
                    .initial_positions
                    .iter()
                    .map(|(c, x, y)| (*c, x - obstacle_x, y - obstacle_y))
                    .collect();

                // Sort normalized positions for consistent filtering
                normalized_positions
                    .sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));

                let normalized_positions_str = normalized_positions
                    .iter()
                    .map(|(c, x, y)| format!("({},{},{})", c, x, y))
                    .collect::<Vec<_>>()
                    .join(",");

                // Get grid info with essential flag
                let grid_info = format!(
                    "GRID:{},{},{},{},{},{},{},{}|INIT:{}|ESSENTIAL:{}|",
                    grid_config.min_x,
                    grid_config.max_x,
                    grid_config.min_y,
                    grid_config.max_y,
                    grid_config.columns,
                    grid_config.rows,
                    grid_config.obstacle_position.0,
                    grid_config.obstacle_position.1,
                    normalized_positions_str,
                    init_config.is_essential
                );

                // Build robots_history from robots_metrics
                // Find the maximum number of positions across all robots
                let max_steps = exp
                    .robots_metrics
                    .iter()
                    .map(|robot| robot.positions.len())
                    .max()
                    .unwrap_or(0);

                // Build history: Vec<Vec<(char, i16, i16)>>
                // Each outer vec element is a step, each inner vec contains all robots at that step
                let mut robots_history: Vec<Vec<(char, i16, i16)>> = Vec::with_capacity(max_steps);

                for step_idx in 0..max_steps {
                    let mut step_positions = Vec::with_capacity(exp.robots_metrics.len());

                    for robot in &exp.robots_metrics {
                        if let Some(position) = robot.positions.get(step_idx) {
                            step_positions.push(*position);
                        }
                    }

                    robots_history.push(step_positions);
                }

                // Format the history as a string
                let history_str = robots_history
                    .iter()
                    .map(|step| {
                        // Format all robots in this step
                        step.iter()
                            .map(|(c, x, y)| format!("({},{},{})", c, x, y))
                            .collect::<Vec<_>>()
                            .join(",")
                    })
                    .collect::<Vec<_>>()
                    .join(";");

                format!("{}{}", grid_info, history_str)
            })
            .collect()
    }
}

pub fn save_algorithm_experiments(
    file_name: &str,
    base_path: &str,
    status: AlgorithmStatus,
    experiment_results: Vec<ExperimentResult>,
    total_activation: usize,
    algo_infos_by_robot_colors: Vec<AlgoInfoByRobotColors>,
) {
    let file_stem = Path::new(file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(file_name);
    let algo_infos = AlgoInfos {
        total_activation,
        by_robot_colors: algo_infos_by_robot_colors,
    };
    let algorithm_experiments = AlgorithmExperiments {
        name: format!("{}_{}", file_stem, status.simple_label()),
        status,
        infos: algo_infos,
        experiments: experiment_results,
    };

    // Create `details` folder if it doesn’t exist
    let details_folder = Path::new(base_path).join("_details");
    fs::create_dir_all(&details_folder).unwrap();

    // Build file path
    let details_file_path = details_folder.join(format!("{}.json", algorithm_experiments.name));

    // Serialize to JSON
    let json_data = serde_json::to_string_pretty(&algorithm_experiments).unwrap();

    // Write JSON to file
    fs::write(&details_file_path, json_data).unwrap();

    //println!("✅ Saved to {}", details_file_path.display());
}
