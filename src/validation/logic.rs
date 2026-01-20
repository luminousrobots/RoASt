// ============================================================================
// ROBOT ALGORITHM VALIDATION SYSTEM
// ============================================================================
//
// This module validates robot algorithms by testing them on different grid configurations.
//
// **How it works:**
// 1. Load algorithm files (.web-algo format)
// 2. Generate test configurations (different grid sizes, robot positions)
// 3. Run each algorithm on each configuration
// 4. Determine if the algorithm works correctly
//
// **Execution Status (what can happen to each test):**
// - ✅ Validated: Algorithm completed successfully
// - ⚠️  ValidatedNoLD: Works but not locally defined (some configs blocked)
// - ❌ Blocked: Algorithm got stuck and couldn't proceed
// - 🔄 Cycle: Algorithm entered an infinite loop
// - ⏱️  Timeout: Algorithm took too long (possible infinite loop)
//
// **Algorithm Status (overall result):**
// Based on all test results, determines if algorithm is reliable
//
// ============================================================================

// External crates
use indicatif::ProgressBar;
use rayon::{prelude::*, vec};

use std::f32::consts::E;
// Standard library
use std::fs::{self, File};
use std::io::{self, BufWriter, Result, Write};
use std::path::{Path, PathBuf};
use std::process::{exit, id};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::classification::logic::classify;
use crate::methodology::configuration::CONFIG;
use crate::methodology::globals::get_execution_root_str;
use crate::modules::algorithm_experiments_modules::algo_info_by_robot_colors::{
    self, AlgoInfoByRobotColors,
};
use crate::modules::algorithm_experiments_modules::algo_infos::AlgoInfos;
use crate::modules::algorithm_experiments_modules::algorithm_experiments::{
    save_algorithm_experiments, AlgorithmExperiments,
};
use crate::modules::algorithm_experiments_modules::algorithm_metrics::AlgorithmMetrics;
use crate::modules::algorithm_experiments_modules::experiment_result::{self, ExperimentResult};
use crate::modules::algorithm_experiments_modules::robot_criteria::{self, RobotCriteria};
use crate::modules::algorithm_snapshot::AlgorithmSnapshot;
use crate::modules::algorithm_stats::AlgorithmStats;
use crate::modules::algorithm_status::AlgorithmStatus;
use crate::modules::color::get_colors;
use crate::modules::config_stats::ConfigStats;
use crate::modules::execution_status::ExecutionStatus;
use crate::modules::exploration_history::ExplorationHistory;
use crate::modules::grid_config::GridConfig;
use crate::modules::grid_experiment::GridExperiment;
use crate::modules::grid_size_generator::generate_grid_definitions;
use crate::modules::init_config::InitConfig;
use crate::modules::position::{self, Position};
use crate::modules::{grid, rule};
// Internal modules

use crate::methodology::view::distribute_abstract_positions;
use crate::modules::{
    blocked_config_summary::BlockedConfigSummary,
    direction::{calculate_movement, rotate_direction, Direction},
    final_rule::FinalRule,
    full_rule::FullRule,
    validation_config::ValidationConfig,
    validation_progress_bars::{create_progress_bars, finish_progress_bars, start_status_updater},
    view::{are_equivalent, rotate_view},
};
use crate::validation::initial_config_generator::generate_initial_configs;
use crate::validation::initial_config_viewer::initial_config_viewer_html;
use crate::validation::logger::{
    create_blocked_summaries_log, log_all_possible_configurations,
    write_algorithm_summary_log, write_validation_summary_log,
};

/// Main validation entry point
/// Validates robot algorithms in the specified directory
pub fn validate(execution_root: &str) {
    println!("🚀 Starting robot algorithm validation...");

    let base_path = get_base_path(execution_root);
    let validation_configs = create_validation_configs(execution_root);
    validate_with_folder_hierarchy(&base_path, &validation_configs);
}

/// Determines the base path for validation based on execution root
fn get_base_path(execution_root: &str) -> String {
    if execution_root.is_empty() {
        "src/data/to_validate".to_string()
    } else {
        format!("{}/Algos", execution_root)
    }
}

/// Creates all validation configurations by combining initial positions with grid definitions
pub fn create_validation_configs(execution_root: &str) -> Vec<GridExperiment> {
    // Generate HTML viewer
    initial_config_viewer_html(
        CONFIG.initial_configurations.clone(),
        CONFIG.visibility_range,
        CONFIG.is_obstacle_opaque,
        CONFIG.number_of_robots,
        format!("{}/initial_configs_viewer.html", execution_root).as_str(),
    )
    .expect("Failed to generate viewer HTML");
    let basic_grid_len = ((CONFIG.number_of_robots as i16 + 1) * CONFIG.visibility_range) * 2 + 1;
    let grid_definitions: Vec<(i16, i16, Vec<(i16, i16)>)> =
        generate_grid_definitions(basic_grid_len);

    // Generate simple text format for grid definitions
    let grid_definitions_path = format!("{}/grid_definitions.txt", get_execution_root_str());
    let mut grid_text = String::new();

    for (row, col, obstacles) in &grid_definitions {
        let obstacles_str: Vec<String> = obstacles
            .iter()
            .map(|(x, y)| format!("[{},{}]", x, y))
            .collect();
        grid_text.push_str(&format!(
            "({}, {}, {{{}}})\n",
            row,
            col,
            obstacles_str.join(", ")
        ));
    }

    std::fs::write(&grid_definitions_path, &grid_text)
        .expect("Failed to write grid definitions to file");
    println!("Grid Definitions written to: {}", grid_definitions_path);

    generate_all_configs_with_positions_indices(
        CONFIG.initial_configurations.clone(),
        grid_definitions,
    )
}

fn parse_activation_level(folder_name: &str) -> Option<i16> {
    folder_name
        .split('_')
        .next()
        .and_then(|s| s.parse::<i16>().ok())
}

fn check_folder_has_validated(snapshot: &AlgorithmSnapshot) -> (bool, bool) {
    let has_validated_ld = !snapshot.validated_ld.is_empty();
    let has_validated_not_ld = !snapshot.validated_not_ld.is_empty();

    (has_validated_ld, has_validated_not_ld)
}

fn validate_with_folder_hierarchy(base_path: &str, list_of_grid_experiment: &[GridExperiment]) {
    let entries = match fs::read_dir(base_path) {
        Ok(entries) => entries,
        Err(_) => {
            eprintln!("Failed to read directory: {}", base_path);
            return;
        }
    };

    let mut folders: Vec<(i16, PathBuf)> = Vec::new();
    let mut has_files = false;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(folder_name) = path.file_name().and_then(|n| n.to_str()) {
                if let Some(activation) = parse_activation_level(folder_name) {
                    folders.push((activation, path));
                }
            }
        } else if path.extension().map_or(false, |ext| ext == "web-algo") {
            has_files = true;
        }
    }

    if has_files && folders.is_empty() {
        println!("Found .web-algo files in base directory. Validating...");
        validate_directory(base_path, list_of_grid_experiment);
        return;
    }

    if !folders.is_empty() {
        folders.sort_by_key(|(activation, _)| *activation);

        println!("Found {} activation folders", folders.len());
        for (activation, path) in &folders {
            println!("  - Activation {}: {:?}", activation, path);
        }

        let mut checked_folders: Vec<(i16, String, bool)> = Vec::new();

        for (activation, folder_path) in folders {
            let folder_path_str = folder_path.to_str().unwrap();
            println!("\n=== Validating activation folder: {} ===", activation);

            let folder_snapshot = validate_directory(folder_path_str, list_of_grid_experiment);

            let (has_validated_ld, has_validated_not_ld) =
                check_folder_has_validated(&folder_snapshot);

            let found_result = has_validated_ld || has_validated_not_ld;
            checked_folders.push((activation, folder_path_str.to_string(), found_result));

            /* if found_result {
                println!(
                    "[OK] Activation {} has validated algorithms (LD: {}, NOT-LD: {})",
                    activation, has_validated_ld, has_validated_not_ld
                );
                println!("Stopping validation at activation {}", activation);
                break;
            } else {
                println!(
                    "[WARN] Activation {} has no validated algorithms. Moving to next level...",
                    activation
                );
            }*/
            println!("Moving to next level...",);
        }

        write_validation_summary_log(base_path, &checked_folders);
    } else {
        println!("No folders or files found to validate");
    }
}

fn validate_directory(
    directory_path: &str,
    list_of_grid_experiment: &[GridExperiment],
) -> AlgorithmSnapshot {
    let algo_files = get_algo_files(directory_path);
    println!(
        "Found {} algorithm files to validate in {}",
        algo_files.len(),
        directory_path
    );

    if algo_files.is_empty() {
        println!("[WARN] No algorithm files found");
        return AlgorithmStats::default().snapshot();
    }

    log_all_possible_configurations(directory_path, list_of_grid_experiment)
        .expect("Failed to log configurations in folder");

    let algo_stats = Arc::new(AlgorithmStats::default());
    let results = Arc::new(Mutex::new(Vec::new()));
    let blocked_summaries = Arc::new(Mutex::new(Vec::<BlockedConfigSummary>::new()));

    run_all_algos(
        &algo_files,
        list_of_grid_experiment,
        &algo_stats,
        &results,
        &blocked_summaries,
        directory_path,
    );

    let algo_snapshot = algo_stats.snapshot();
    let validated_ld_algo = algo_snapshot.validated_ld.len();
    let validated_not_ld_algo = algo_snapshot.validated_not_ld.len();
    let blocked_algo = algo_snapshot.blocked.len();
    let cyclic_algo = algo_snapshot.cyclic.len();
    let timeout_algo = algo_snapshot.timeout.len();
    let total_count: usize = algo_files.len();

    write_algorithm_summary_log(directory_path, &results, &algo_snapshot, total_count);

    println!(
        "Validation results written to '{}'",
        format!("{}/_validation_results.log", directory_path)
    );
    println!(
        "Summary: {} validated (LD), {} validated (NOT-LD), {} blocked, {} cyclic, {} timeout",
        validated_ld_algo, validated_not_ld_algo, blocked_algo, cyclic_algo, timeout_algo
    );

    create_blocked_summaries_log(
        directory_path,
        &blocked_summaries,
        &algo_files,
        list_of_grid_experiment,
    );

    // Run classification after validation
    classify(
        format!("{}/_details", directory_path).as_str(),
        directory_path,
    );

    algo_snapshot
}

// ============================================================================
// PARALLEL VALIDATION EXECUTION
// ============================================================================

fn run_all_algos(
    algo_files: &[(String, String)],
    list_of_grid_experiment: &[GridExperiment],
    algo_stats: &Arc<AlgorithmStats>,
    results: &Arc<Mutex<Vec<String>>>,
    blocked_summaries: &Arc<Mutex<Vec<BlockedConfigSummary>>>,
    base_path: &str,
) {
    // Create progress bars
    let progress_bars = create_progress_bars(algo_files.len(), list_of_grid_experiment.len());

    // Start status updater thread with ALGORITHM-LEVEL counters (not config-level)
    // We want to see algorithm progress during execution, not individual config counts
    start_status_updater(
        progress_bars.algo.clone(),
        progress_bars.status.clone(),
        Arc::clone(algo_stats),
        base_path.to_string(),
        progress_bars.start_time,
    );

    // Process algorithms in parallel
    let pb_algo = progress_bars.algo.clone();
    let pb_config = progress_bars.config.clone();

    algo_files
        .par_iter()
        .enumerate()
        .for_each(|(index, (algo, file_name))| {
            process_algo(
                index,
                algo,
                file_name,
                list_of_grid_experiment,
                algo_stats,
                results,
                blocked_summaries,
                &pb_config,
                base_path,
            );
            pb_algo.inc(1);
        });

    // Get final algorithm-level counts
    let final_snapshot: AlgorithmSnapshot = algo_stats.snapshot();

    // Finish progress bars with final stats
    finish_progress_bars(
        &progress_bars,
        final_snapshot.validated_ld.len(),
        final_snapshot.validated_not_ld.len(),
        final_snapshot.blocked.len(),
        final_snapshot.cyclic.len(),
        final_snapshot.timeout.len(),
        algo_files.len(),
        base_path,
    );
}

// ============================================================================
// SINGLE ALGORITHM PROCESSING
// ============================================================================

fn process_algo(
    index: usize,
    algo: &str,
    file_name: &str,
    list_of_grid_experiment: &[GridExperiment],
    algo_stats: &Arc<AlgorithmStats>,
    results: &Arc<Mutex<Vec<String>>>,
    blocked_summaries: &Arc<Mutex<Vec<BlockedConfigSummary>>>,
    pb_config: &ProgressBar,
    base_path: &str,
) {
    let config_stats = Arc::new(ConfigStats::default());

    let (final_rules, visibility) = calculate_final_rules(algo);

    let sim_results: Vec<(ExecutionStatus, ExperimentResult)> = list_of_grid_experiment
        .par_iter()
        .enumerate()
        .map(|(i, grid_experiment)| {
            let result = simulate_exploration(i, grid_experiment, &final_rules, visibility);
            pb_config.inc(1);
            result
        })
        .collect();

    // Separate into two vectors
    let (statuses, experiment_results): (Vec<_>, Vec<_>) = sim_results.into_iter().unzip();

    // Record blocked configurations
    for (config_index, (status, experiment)) in
        statuses.iter().zip(experiment_results.iter()).enumerate()
    {
        if *status == ExecutionStatus::Blocked || *status == ExecutionStatus::BlockedNotEssential {
            record_blocked_config(
                blocked_summaries,
                file_name,
                config_index,
                &list_of_grid_experiment[config_index],
            );
        }
    }

    let metrics = AlgorithmMetrics::from_outcomes(&statuses);
    let status: AlgorithmStatus = determine_algorithm_status(&metrics);

    // Update counters based on final status
    update_algorithm_counters(status, algo_stats, file_name);
    let algo_summary = format_algorithm_result(index, file_name, status, &metrics, &statuses);

    //TODO:
    let (algo_infos_by_robot_colors, total_activation) =
        calculate_algo_infos_by_robot_colors(&final_rules);

    if status == AlgorithmStatus::Validated {
        save_algorithm_experiments(
            &file_name,
            base_path,
            status,
            experiment_results,
            total_activation,
            algo_infos_by_robot_colors,
        );
    }
    results.lock().unwrap().push(algo_summary);
}

pub fn calculate_algo_infos_by_robot_colors(
    rules: &[FinalRule],
) -> (Vec<AlgoInfoByRobotColors>, usize) {
    let mut infos_by_colors: Vec<AlgoInfoByRobotColors> = vec![];
    let colors: Vec<char> = get_colors(&CONFIG.all_color_letters.to_vec(), CONFIG.number_of_colors);
    let mut total_activation = 0;
    for color in colors {
        let (color_activations, movement_activations, rule_count, idle_count, opacity_count) =
            claculate_activations_by_color(rules, color);
        let algo_info_by_robot_colors = AlgoInfoByRobotColors {
            robot_color: color,
            total_activation: color_activations + movement_activations,
            color_activations,
            movement_activations,
            rules_count: rule_count,
            idle_rules_count: idle_count,
            opacity_rule_count: opacity_count,
        };
        infos_by_colors.push(algo_info_by_robot_colors);
        total_activation += color_activations + movement_activations;
    }

    (infos_by_colors, total_activation)
}
fn claculate_activations_by_color(
    rules: &[FinalRule],
    color: char,
) -> (usize, usize, usize, usize, usize) {
    let mut color_activations = 0;
    let mut movement_activations = 0;
    let mut rule_count = 0;
    let mut idle_count = 0;
    let mut opacity_count = 0;

    for rule in rules {
        if rule.color == color {
            rule_count += 1;
            if rule.direction == Direction::Idle {
                idle_count += 1;
            } else {
                color_activations += 1;
                if rule.color == rule.view[0].0 {
                    movement_activations += 1;
                }
            }
            if rule.view.iter().any(|(c, _, _)| *c == 'X') {
                opacity_count += 1;
            }
        }
    }

    (
        color_activations,
        movement_activations,
        rule_count,
        idle_count,
        opacity_count,
    )
}

fn simulate_exploration(
    experiment_id: usize,
    grid_experiment: &GridExperiment,
    final_rules: &[FinalRule],
    visibility: i16,
) -> (ExecutionStatus, ExperimentResult) {
    // Initialize robot metrics
    let mut color_activations = initialize_counters(&grid_experiment);
    let mut movement_activations = initialize_counters(&grid_experiment);
    let mut rules_count = initialize_counters(&grid_experiment);
    let mut idle_rules_count = initialize_counters(&grid_experiment);
    let mut activations_per_step: Vec<usize> = vec![]; // Track activations at each step

    let mut robots_history: Vec<Vec<(char, i16, i16)>> =
        vec![grid_experiment.init_config.initial_positions.clone()];
    let mut exploration_history = ExplorationHistory::new(&grid_experiment.grid_config);
    let mut steps = 0;
    let mut cycle_len = 0;

    loop {
        steps += 1;

        // Try to make a move
        if simulate_step(
            &mut robots_history,
            final_rules,
            grid_experiment,
            visibility,
            &mut color_activations,
            &mut movement_activations,
            &mut rules_count,
            &mut idle_rules_count,
            &mut activations_per_step,
        ) {
            let experiment_result = calculate_experiment_result(
                experiment_id,
                grid_experiment,
                &robots_history,
                &color_activations,
                &movement_activations,
                &rules_count,
                &idle_rules_count,
                steps,
                0,
                &[],
            );
            if (grid_experiment.init_config.is_essential) {
                return (ExecutionStatus::Blocked, experiment_result);
            } else {
                return (ExecutionStatus::BlockedNotEssential, experiment_result);
            }
        }

        // Update exploration history
        let last_state = robots_history.last().unwrap();
        exploration_history.set_positions(last_state, &grid_experiment.grid_config);

        // Check if we've completed exploration
        cycle_len = is_exploration_finished_(&robots_history);
        if cycle_len > 0 {
            break;
        }

        /*  // Check for timeout
        if steps > MAX_EXPLORATION_STEPS {
            eprintln!(
                "⚠️  Exploration timeout after {} steps",
                MAX_EXPLORATION_STEPS
            );

            let experiment_result = calculate_experiment_result(
                experiment_id,
                grid_experiment,
                &robots_history,
                &color_activations,
                &movement_activations,
                &rules_count,
                &idle_rules_count,
                steps,
                0,
                &[],
            );
            return (ExecutionStatus::Timeout, experiment_result);
        }*/
    }

    let experiment_result = calculate_experiment_result(
        experiment_id,
        grid_experiment,
        &robots_history,
        &color_activations,
        &movement_activations,
        &rules_count,
        &idle_rules_count,
        steps,
        cycle_len,
        &activations_per_step,
    );
    // Determine final status
    if exploration_history.is_fully_explored() {
        (ExecutionStatus::Validated, experiment_result)
    } else {
        (ExecutionStatus::Cycle, experiment_result)
    }
}

fn initialize_counters(grid_experiment: &GridExperiment) -> Vec<usize> {
    grid_experiment
        .init_config
        .initial_positions
        .iter()
        .map(|_| 0)
        .collect()
}
fn calculate_experiment_result(
    experiment_id: usize,
    grid_experiment: &GridExperiment,
    robots_history: &Vec<Vec<(char, i16, i16)>>,
    color_activations: &Vec<usize>,
    movement_activations: &Vec<usize>,
    rule_count: &Vec<usize>,
    idle_rules_count: &Vec<usize>,
    steps: usize,
    cycle_len: usize,
    activations_per_step: &[usize],
) -> ExperimentResult {
    //i need to modify this part
    let mut robots_metrics: Vec<RobotCriteria> = vec![];
    let init_positions = &grid_experiment.init_config.initial_positions;

    for (position_index, (c, x, y)) in init_positions.iter().enumerate() {
        if (*c != CONFIG.obstacle) {
            let col_act = color_activations[position_index];
            let mov_act = movement_activations[position_index];
            let rule_count = rule_count[position_index];
            let idle_count = idle_rules_count[position_index];
            let total_act = col_act + mov_act;
            let robot_history = robots_history
                .iter()
                .map(|step| step[position_index])
                .collect();
            let robot_criteria = RobotCriteria {
                robot_id: position_index,
                robot_position: (*c, *x, *y),
                total_activation: total_act,
                color_activations: col_act,
                movement_activations: mov_act,
                rule_count: rule_count,
                idle_rule_count: idle_count,
                positions: robot_history,
            };
            robots_metrics.push(robot_criteria);
        }
    }

    // Calculate total activation in cycle by summing the last cycle_len steps
    let total_activation_in_cycle: usize =
        if cycle_len > 0 && activations_per_step.len() >= cycle_len {
            activations_per_step.iter().rev().take(cycle_len).sum()
        } else {
            0
        };

    let total_activation: usize = activations_per_step.iter().sum();

    ExperimentResult {
        experiment_id,
        grid_experiment: grid_experiment.clone(),
        status: ExecutionStatus::Validated,
        steps_taken: steps,
        cycle_len: cycle_len,
        total_activation_in_cycle,
        total_activation,
        robots_metrics: robots_metrics.clone(),
    }
}

/// Records a blocked configuration for later analysis
fn record_blocked_config(
    blocked_summaries: &Arc<Mutex<Vec<BlockedConfigSummary>>>,
    file_name: &str,
    config_index: usize,
    grid_experiment: &GridExperiment,
) {
    if let Ok(mut summaries) = blocked_summaries.lock() {
        summaries.push(BlockedConfigSummary::new_with_flag(
            file_name.to_string(),
            config_index,
            grid_experiment.init_config.current_position_index,
            grid_experiment.init_config.is_essential,
        ));
    }
}

fn config_status_label(status: &ExecutionStatus) -> &'static str {
    status.to_string()
}

/// Determines algorithm status using priority rules
fn determine_algorithm_status(metrics: &AlgorithmMetrics) -> AlgorithmStatus {
    if metrics.timeout > 0 {
        AlgorithmStatus::Timeout
    } else if metrics.cyclic > 0 {
        AlgorithmStatus::Cyclic
    } else if metrics.blocked > 0 {
        AlgorithmStatus::Blocked
    } else if metrics.validated_not_ld > 0 {
        AlgorithmStatus::ValidatedNotLd
    } else if metrics.validated_ld > 0 {
        AlgorithmStatus::Validated
    } else {
        AlgorithmStatus::Unknown
    }
}

/// Updates algorithm counters based on the determined status
fn update_algorithm_counters(
    status: AlgorithmStatus,
    algo_stats: &Arc<AlgorithmStats>,
    algo_name: &str,
) {
    algo_stats.insert(status, algo_name);
}
/// Formats the algorithm result for display
fn format_algorithm_result(
    index: usize,
    file_name: &str,
    algo_status: AlgorithmStatus,
    metrics: &AlgorithmMetrics,
    config_outcomes: &[ExecutionStatus],
) -> String {
    let mut result = format!(
        "{}) {} ................... {} (✅{}  ⚠️{}  ❌{}  🔄{}  ⏱️{})\n",
        index,
        file_name,
        algo_status.label(),
        metrics.validated_ld,
        metrics.validated_not_ld,
        metrics.blocked,
        metrics.cyclic,
        metrics.timeout
    );

    // Add details for each configuration (optional - can be commented out for cleaner output)
    for (i, status) in config_outcomes.iter().enumerate() {
        result.push_str(&format!(
            "    • Config {} .................... {} - {}\n",
            i,
            config_status_label(status),
            status.description()
        ));
    }

    result
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

pub fn is_exploration_finished(robots_history: &Vec<Vec<(char, i16, i16)>>) -> bool {
    if robots_history.len() < 2 {
        return false;
    }
    for state in robots_history.iter().skip(1) {
        if are_equivalent(state, &robots_history[0]) {
            return true;
        }
    }

    for state in robots_history.iter().rev().skip(1) {
        if are_equivalent(state, robots_history.last().unwrap()) {
            return true;
        }
    }

    false
}

pub fn is_exploration_finished_(robots_history: &Vec<Vec<(char, i16, i16)>>) -> usize {
    if robots_history.len() < 2 {
        return 0;
    }

    let last_index = robots_history.len() - 1;
    let last_state = &robots_history[last_index];

    // Quick check: first == last
    if are_equivalent(last_state, &robots_history[0]) {
        return last_index; // distance between first and last
    }

    // Otherwise, check backward for any repeated state
    let last_index = robots_history.len() - 1;
    for (i, state) in robots_history.iter().enumerate().rev().skip(1) {
        if are_equivalent(state, &robots_history[last_index]) {
            return last_index - i; // Distance from the last state
        }
    }

    0 // No equivalent state found
}

// These helper functions have been removed to simplify the code.
// The exploration history functionality is handled by the ExplorationHistory module.

fn simulate_step(
    robots_history: &mut Vec<Vec<(char, i16, i16)>>,
    final_rules: &[FinalRule],
    grid_experiment: &GridExperiment,
    visibility: i16,
    color_activations: &mut Vec<usize>,
    movement_activations: &mut Vec<usize>,
    rules_count: &mut Vec<usize>,
    idle_rules_count: &mut Vec<usize>,
    activations_per_step: &mut Vec<usize>,
) -> bool {
    let mut queue: Vec<(char, i16, i16)> = vec![];
    let mut is_blocked = true;
    let mut total_activations_in_step: usize = 0;

    if let Some(last_state) = robots_history.last() {
        for (i, robot) in last_state.iter().enumerate() {
            // Clone all other robots except the one at index i
            let mut other_robots = last_state.clone();
            other_robots.remove(i);
            //    println!("Robot: {:?}", robot);

            let mut robot_view = calculate_view_with_walls(
                *robot,
                &other_robots,
                visibility,
                &grid_experiment.grid_config,
            );
            //display_view(&robot_view, &visibility);
            if CONFIG.opacity {
                distribute_abstract_positions(&mut robot_view, visibility);
            }

            //     println!("Robot view: {:?}", robot_view);
            if let Some((dir, color)) = find_matched_rule(&robot_view, final_rules) {
                let (x, y) = calculate_movement(&dir, &robot.1, &robot.2);

                if x != robot.1 || y != robot.2 {
                    movement_activations[i] += 1;
                    total_activations_in_step += 1;
                }
                if color != robot.0 {
                    color_activations[i] += 1;
                    total_activations_in_step += 1;
                }

                rules_count[i] += 1;

                if dir == Direction::Idle {
                    idle_rules_count[i] += 1;
                }

                queue.push((color, x, y));
                is_blocked = false;
            } else {
                queue.push(*robot);
            }
        }
        robots_history.push(queue);
    }
    activations_per_step.push(total_activations_in_step);
    is_blocked
}

pub fn find_matched_rule(
    robot_view: &Vec<(char, i16, i16)>,
    final_rules: &[FinalRule],
) -> Option<(Direction, char)> {
    let rotations_angles = vec![0, 90, 180, 270];
    let suspected_final_rules = get_suspected_final_rules(&robot_view[0].0, final_rules);
    //  println!("Suspected rules len: {:?}", suspected_final_rules.len());
    /*  for (i, rule) in suspected_final_rules.iter().enumerate() {
        println!("-----------------Suspected rule {}----------------: ", i);
        print_final_rule(rule, visibility);
    }*/
    let mut matched_rule: Option<(Direction, char)> = None;
    let mut match_count = 0;

    for rule in suspected_final_rules {
        let rule_view = rule.view.clone();
        let rule_direction = rule.direction.clone();
        let rule_color = rule.color;

        for &angle in rotations_angles.iter() {
            let rotated_view = rotate_view(&rule_view, angle);
            let rotated_direction = rotate_direction(&rule_direction, angle);

            if are_equivalent(robot_view, &rotated_view) {
                match_count += 1;

                if match_count > 1 {
                    panic!("Multiple matched rules found for the given robot view!");
                }

                matched_rule = Some((rotated_direction, rule_color));
            }
        }
    }

    matched_rule
}

fn get_suspected_final_rules(robot_marker: &char, final_rules: &[FinalRule]) -> Vec<FinalRule> {
    final_rules
        .iter()
        .enumerate()
        .filter_map(|(_, rule)| {
            if rule.view[0].0 != *robot_marker {
                return None;
            }
            Some(rule.clone()) // Return the rule itself
        })
        .collect()
}

pub fn calculate_view_with_walls(
    robot: (char, i16, i16),
    other_robots: &Vec<(char, i16, i16)>,
    visibility: i16,
    grid_config: &GridConfig,
) -> Vec<(char, i16, i16)> {
    let (ch, robot_x, robot_y) = robot; // Extract the character and coordinates of the robot
    let mut robots_view: Vec<(char, i16, i16)> = vec![(ch, 0, 0)];

    /*  let mut i = 0;
        while i < other_robots.len() {
            let temp_ch = other_robots[i].0;
            let temp_x = other_robots[i].1;
            let temp_y = other_robots[i].2;

            robots_view.push((temp_ch, temp_x - robot_x, temp_y - robot_y));

            i += 1;
        }
    */
    for &(ch, x, y) in other_robots {
        // Calculate the relative position
        let a = x - robot_x;
        let b = y - robot_y;

        // Check if the robot is within the visibility range
        if a.abs() + b.abs() <= visibility {
            robots_view.push((ch, a, b));
        }
    }

    // Check for positions within visibility and on the bounds
    for j in -visibility..=visibility {
        for i in -visibility..=visibility {
            if i.abs() + j.abs() <= visibility {
                let global_x = robot_x + i;
                let global_y = robot_y + j;

                // Check if the position is on the bounds
                if global_x == grid_config.min_x
                    || global_x == grid_config.max_x
                    || global_y == grid_config.min_y
                    || global_y == grid_config.max_y
                {
                    robots_view.push(('W', i, j)); // Add 'W' for positions on the bounds
                }
            }
        }
    }

    robots_view
}

pub fn calculate_final_rules(content: &str) -> (Vec<FinalRule>, i16) {
    let (rules, visibility_range) = extract_rules_from_content(content);
    // println!("Visibility Range: {:?}", visibility_range);
    //println!("Validating algorithm: {} rules found", rules.len());
    let final_rules = convert_full_rules_to_final_rules(&rules, visibility_range);
    /*for rule in &final_rules {
        print_final_rule(&rule, visibility_range);
    }*/
    (final_rules, visibility_range)
}

fn extract_rules_from_content(content: &str) -> (Vec<FullRule>, i16) {
    let content = content
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");

    /*  println!(
        "Content after removing comments and blank lines:\n{}",
        content
    );*/
    let mut rules = Vec::new();
    let mut visibility_range: i16 = 0;

    let parts: Vec<&str> = content.split("****** RULES ******").collect();

    if parts.len() < 2 {
        eprintln!("No RULES section found.");
        return (rules, visibility_range);
    }

    // Extract visibilityRange from the rules section
    for line in parts[0].lines() {
        if line.trim_start().starts_with("visibilityRange:") {
            if let Some(value) = line.split(':').nth(1) {
                visibility_range = value.trim().parse::<i16>().unwrap_or(0);
            }
        }
    }

    let rules_section = parts[1];

    let rule_blocks: Vec<&str> = rules_section
        .split("\n\n")
        .filter(|block| block.contains("->"))
        .collect();

    for block in rule_blocks {
        let lines: Vec<&str> = block
            .lines()
            .map(|line| line.trim_end())
            .filter(|line| !line.starts_with('#') && !line.is_empty())
            .collect();

        if lines.len() < 3 {
            continue;
        }

        let mut view: Vec<Vec<char>> = vec![];
        let mut direction: Direction = Direction::Idle;
        let mut color: char = ' ';

        for line in lines.iter() {
            if line.contains("->") {
                let parts: Vec<&str> = line.split("->").collect();
                if parts.len() == 2 {
                    let left = parts[0].trim();
                    let right = parts[1].trim();
                    let view_line = left.to_string();

                    view.push(view_line.chars().collect());

                    let right_parts: Vec<&str> = right.split(',').collect();
                    if right_parts.len() == 2 {
                        direction = Direction::from_str(right_parts[0].trim());
                        color = right_parts[1].trim().chars().next().unwrap_or(' ');
                    }
                }
            } else {
                view.push(line.chars().collect());
            }
        }

        rules.push(FullRule {
            view,
            direction,
            color,
        });
    }

    (rules, visibility_range)
}

pub fn get_algo_files(path: &str) -> Vec<(String, String)> {
    let mut files_content = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "web-algo" {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
                            files_content.push((content, file_name.to_string()));
                        }
                    }
                }
            }
        }
    }

    files_content
}

pub fn convert_full_rules_to_final_rules(rules: &[FullRule], visibility: i16) -> Vec<FinalRule> {
    rules
        .iter()
        .map(|rule| convert_full_rule_to_final_rule(rule, visibility))
        .collect()
}
pub fn convert_full_rule_to_final_rule(rule: &FullRule, visibility: i16) -> FinalRule {
    let center = visibility; // Center of the grid
    let mut view = vec![];

    for (y, row) in rule.view.iter().enumerate() {
        for (x, &ch) in row.iter().enumerate() {
            // Skip characters that are '.' or ' '
            if ch != '.' && ch != ' ' {
                // Adjust coordinates to match the 2D plane interpretation
                let adjusted_x = x as i16 - center;
                let adjusted_y = center - y as i16;

                // Insert at the start if adjusted_x == 0 and adjusted_y == 0
                if adjusted_x == 0 && adjusted_y == 0 {
                    view.insert(0, (ch, adjusted_x, adjusted_y));
                } else {
                    view.push((ch, adjusted_x, adjusted_y));
                }
            }
        }
    }

    FinalRule {
        view,
        direction: rule.direction.clone(),
        color: rule.color,
    }
}
// Update generate_all_configs_with_positions_indices to include essential flag
pub fn generate_all_configs_with_positions_indices(
    list_of_initial_positions: Vec<(Vec<(char, i16, i16)>, bool)>, // bool indicates if essential
    grid_definitions: Vec<(i16, i16, Vec<(i16, i16)>)>,
) -> Vec<GridExperiment> {
    // NEW: Added bool to return type
    // Validate that all orientations include OBSTACLE at (0, 0)
    for (i, (pos_list, _)) in list_of_initial_positions.iter().enumerate() {
        let has_o = pos_list
            .iter()
            .any(|&(ch, x, y)| ch == CONFIG.obstacle && x == 0 && y == 0);
        //  println!("indx: {}: {:?}", i, pos_list);
        if !has_o {
            panic!(
                "Initial position set {} is missing robot OBSTACLE at (0, 0)",
                i
            );
        }
    }

    // Configuration counter (for potential debugging)
    let mut configs: Vec<GridExperiment> = vec![];
    let mut id_counter: usize = 0; // <--- incrementing ID

    for (cols, rows, obstacles) in grid_definitions {
        let min_x = 0;
        let max_x = cols + 1;
        let min_y = 0;
        let max_y = rows + 1;

        for (obs_x, obs_y) in obstacles.iter() {
            for (initial_positions_index, (initial_set, is_essential)) in
                list_of_initial_positions.iter().enumerate()
            {
                let mut shifted_initial_positions = vec![];

                for &(ch, x, y) in initial_set.iter() {
                    shifted_initial_positions.push((ch, x + *obs_x, y + *obs_y));
                }

                let new_config = ValidationConfig {
                    initial_positions: shifted_initial_positions.clone(),
                    min_x,
                    max_x,
                    min_y,
                    max_y,
                };

                /*   println!(
                     "---------------New Config {} (Essential: {})----------------",
                     new_config_count, is_essential
                 );
                 new_config_count += 1;
                // print_grid_from_config(&new_config);*/

                configs.push(GridExperiment {
                    id: id_counter,
                    grid_config: GridConfig {
                        columns: cols,
                        rows: rows,
                        min_x: min_x,
                        max_x: max_x,
                        min_y: min_y,
                        max_y: max_y,
                        obstacle_position: (*obs_x, *obs_y),
                    },
                    init_config: InitConfig {
                        current_position_index: initial_positions_index,
                        initial_positions: shifted_initial_positions.clone(),
                        is_essential: *is_essential,
                    },
                });
                id_counter += 1;
                // NEW: Include essential flag
            }
        }
    }

    configs
}

fn print_grid(cols: usize, rows: usize, obstacle: (usize, usize)) {
    println!(
        "\nGrid ({}x{}) with obstacle at {:?}:\n",
        cols, rows, obstacle
    );

    let width = cols + 2;
    let height = rows + 2;
    let (obs_x, obs_y) = obstacle;

    for y in (0..height).rev() {
        print!("{:>2} ", y);
        for x in 0..width {
            if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                print!("W ");
            } else if x == obs_x && y == obs_y {
                print!("X ");
            } else {
                print!(". ");
            }
        }
        println!();
    }

    print!("   ");
    for x in 0..width {
        print!("{} ", x);
    }
    println!("\n");
}
fn print_grid_from_config(config: &ValidationConfig) {
    println!(
        "\nGrid ({}x{}) with positions:",
        config.max_x - config.min_x - 1,
        config.max_y - config.min_y - 1
    );

    let width = (config.max_x - config.min_x + 1) as usize;
    let height = (config.max_y - config.min_y + 1) as usize;

    let mut grid = vec![vec!['.'; width]; height];

    // Fill boundaries with 'W' for walls
    for y in 0..height {
        grid[y][0] = 'W';
        grid[y][width - 1] = 'W';
    }
    for x in 0..width {
        grid[0][x] = 'W';
        grid[height - 1][x] = 'W';
    }

    // Place initial positions
    for &(ch, x, y) in &config.initial_positions {
        let gx = (x - config.min_x) as usize;
        let gy = (config.max_y - y) as usize;
        if gy < height && gx < width {
            grid[gy][gx] = ch;
        }
    }

    for (y_index, row) in grid.iter().enumerate() {
        print!("{:>2} ", config.max_y - y_index as i16);
        for &cell in row {
            print!("{} ", cell);
        }
        println!();
    }

    print!("   ");
    for x in config.min_x..=config.max_x {
        print!("{} ", x);
    }
    println!("\n");
}

fn print_config_debug(new_config: &ValidationConfig) {
    let positions_str: String = new_config
        .initial_positions
        .iter()
        .map(|(ch, x, y)| format!("('{}', {}, {})", ch, x, y))
        .collect::<Vec<_>>()
        .join(" ");

    // Try to find the obstacle
    let obs = new_config
        .initial_positions
        .iter()
        .find(|&&(ch, _, _)| ch == CONFIG.obstacle)
        .map(|&(_, x, y)| (x, y))
        .unwrap_or((0, 0));

    let width = new_config.max_x - new_config.min_x - 1;
    let height = new_config.max_y - new_config.min_y - 1;

    println!("\n=== ValidationConfig ===");
    println!("({}x{}) with obs at ({},{})", width, height, obs.0, obs.1);
    println!("Initial Positions: {}", positions_str);
    println!(
        "Boundaries: ({}, {}, {}, {})",
        new_config.min_x, new_config.min_y, new_config.max_x, new_config.max_y
    );
    println!("====================\n");
}

/*
fn print_config_debug(new_config: &ValidationConfig) {
    println!("\n=== ValidationConfig ===");
    println!("Initial Positions:");
    for (ch, x, y) in &new_config.initial_positions {
        println!("  - ('{}', {}, {})", ch, x, y);
    }

    println!("Boundaries:");
    println!("  x: {} to {}", new_config.min_x, new_config.max_x);
    println!("  y: {} to {}", new_config.min_y, new_config.max_y);
    println!("========================\n");
}
*/

/// Validates algorithms in a single folder (no hierarchy traversal)
pub fn validate_single_folder(folder_path: &str)->AlgorithmSnapshot {
    println!("🔍 Validating single folder: {}", folder_path);

    let validation_configs = create_validation_configs(folder_path);
    let algorithm_snapshot = validate_directory(folder_path, &validation_configs);


    algorithm_snapshot
}
