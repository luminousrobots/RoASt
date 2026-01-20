use crate::methodology::configuration::CONFIG;
use crate::modules::algorithm_snapshot;
use crate::modules::{
    algorithm_snapshot::AlgorithmSnapshot, blocked_config_summary::BlockedConfigSummary,
    grid_experiment::GridExperiment, validation_config::ValidationConfig,
};
use std::io::{self, BufWriter, Result, Write};
use std::{
    fs::{self, File},
    path::Path,
    sync::{Arc, Mutex},
};

pub fn log_all_possible_configurations(base_path: &str, configs: &[GridExperiment]) -> Result<()> {
    // Create directory if it doesn't exist
    if let Some(parent) = Path::new(base_path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir_all(base_path)?;

    let log_path = Path::new(&base_path).join("_all_possible_configurations.log");
    let file = File::create(log_path)?;
    let mut writer = BufWriter::new(file);

    for (config_id, exp) in configs.iter().enumerate() {
        write_config_debug(&exp, config_id, &mut writer)?;
        write_grid_from_config(&exp, &mut writer)?;
        writeln!(writer, "\n----------------------------------------\n")?;
    }

    Ok(())
}

fn write_config_debug<W: Write>(
    grid_experiment: &GridExperiment,
    config_id: usize,
    writer: &mut W,
) -> Result<()> {
    let positions_str: String = grid_experiment
        .init_config
        .initial_positions
        .iter()
        .map(|(ch, x, y)| format!("('{}', {}, {})", ch, x, y))
        .collect::<Vec<_>>()
        .join(" ");

    let obs = grid_experiment
        .init_config
        .initial_positions
        .iter()
        .find(|&&(ch, _, _)| ch == CONFIG.obstacle)
        .map(|&(_, x, y)| (x, y))
        .unwrap_or((0, 0));

    let width = grid_experiment.grid_config.max_x - grid_experiment.grid_config.min_x - 1;
    let height = grid_experiment.grid_config.max_y - grid_experiment.grid_config.min_y - 1;

    writeln!(writer, "=== ValidationConfig {} ===", config_id)?;
    writeln!(
        writer,
        "({}x{}) with obs at ({},{})",
        width, height, obs.0, obs.1
    )?;
    writeln!(writer, "Initial Positions: {}", positions_str)?;
    writeln!(
        writer,
        "Boundaries: ({}, {}, {}, {})",
        grid_experiment.grid_config.min_x,
        grid_experiment.grid_config.min_y,
        grid_experiment.grid_config.max_x,
        grid_experiment.grid_config.max_y
    )?;
    writeln!(writer, "========================")?;
    Ok(())
}

fn write_grid_from_config<W: Write>(
    grid_experiment: &GridExperiment,
    writer: &mut W,
) -> Result<()> {
    writeln!(
        writer,
        "\nGrid ({}x{}) with positions:",
        grid_experiment.grid_config.max_x - grid_experiment.grid_config.min_x - 1,
        grid_experiment.grid_config.max_y - grid_experiment.grid_config.min_y - 1
    )?;

    let width =
        (grid_experiment.grid_config.max_x - grid_experiment.grid_config.min_x + 1) as usize;
    let height =
        (grid_experiment.grid_config.max_y - grid_experiment.grid_config.min_y + 1) as usize;

    let mut grid = vec![vec!['.'; width]; height];

    for y in 0..height {
        grid[y][0] = 'W';
        grid[y][width - 1] = 'W';
    }
    for x in 0..width {
        grid[0][x] = 'W';
        grid[height - 1][x] = 'W';
    }

    for &(ch, x, y) in &grid_experiment.init_config.initial_positions {
        let gx = (x - grid_experiment.grid_config.min_x) as usize;
        let gy = (grid_experiment.grid_config.max_y - y) as usize;
        if gy < height && gx < width {
            grid[gy][gx] = ch;
        }
    }

    for (y_index, row) in grid.iter().enumerate() {
        write!(
            writer,
            "{:>2} ",
            grid_experiment.grid_config.max_y - y_index as i16
        )?;
        for &cell in row {
            write!(writer, "{} ", cell)?;
        }
        writeln!(writer)?;
    }

    write!(writer, "   ")?;
    for x in grid_experiment.grid_config.min_x..=grid_experiment.grid_config.max_x {
        write!(writer, "{} ", x)?;
    }
    writeln!(writer, "\n")?;

    Ok(())
}

pub fn create_blocked_summaries_log(
    base_path: &str,
    blocked_summaries: &Arc<Mutex<Vec<BlockedConfigSummary>>>,
    algo_files: &[(String, String)],
    grid_experiments: &[GridExperiment],
) {
    let log_path = Path::new(base_path).join("_blocked_summaries.log");
    let mut log_file =
        File::create(&log_path).expect("Unable to create blocked summaries log file");

    writeln!(log_file, "BLOCKED SUMMARIES BY ALGORITHM").expect("Unable to write to log file");
    writeln!(log_file, "=====================================")
        .expect("Unable to write to log file");
    writeln!(log_file, "").expect("Unable to write to log file");

    let summaries = blocked_summaries.lock().unwrap();

    // Separate essential and non-essential blocked positions by algorithm
    let mut algo_essential_blocked: std::collections::HashMap<
        String,
        std::collections::HashSet<usize>,
    > = std::collections::HashMap::new();

    let mut algo_not_ld_blocked: std::collections::HashMap<
        String,
        std::collections::HashSet<usize>,
    > = std::collections::HashMap::new();

    for summary in summaries.iter() {
        if summary.is_essential {
            // Essential config blocked
            let entry = algo_essential_blocked
                .entry(summary.algorithm_name.clone())
                .or_insert_with(std::collections::HashSet::new);
            entry.insert(summary.blocked_initial_positions_index);
        } else {
            // Non-essential config blocked (NOT-LD)
            let entry = algo_not_ld_blocked
                .entry(summary.algorithm_name.clone())
                .or_insert_with(std::collections::HashSet::new);
            entry.insert(summary.blocked_initial_positions_index);
        }
    }

    // Write results for each algorithm
    for (algo_index, (_, file_name)) in algo_files.iter().enumerate() {
        writeln!(log_file, "{}) {}", algo_index, file_name).expect("Unable to write to log file");

        // Show essential blocked positions
        if let Some(essential_blocked) = algo_essential_blocked.get(file_name) {
            if !essential_blocked.is_empty() {
                let mut sorted_positions: Vec<usize> = essential_blocked.iter().cloned().collect();
                sorted_positions.sort();
                writeln!(
                    log_file,
                    "   → [ESSENTIAL] Blocked position IDs: {:?}",
                    sorted_positions
                )
                .expect("Unable to write to log file");
                writeln!(
                    log_file,
                    "   → [ESSENTIAL] Total blocked positions: {}",
                    sorted_positions.len()
                )
                .expect("Unable to write to log file");
            }
        }

        // Show non-essential blocked positions (NOT-LD)
        if let Some(not_ld_blocked) = algo_not_ld_blocked.get(file_name) {
            if !not_ld_blocked.is_empty() {
                let mut sorted_positions: Vec<usize> = not_ld_blocked.iter().cloned().collect();
                sorted_positions.sort();
                writeln!(
                    log_file,
                    "   → [NOT-LD] Blocked position IDs: {:?}",
                    sorted_positions
                )
                .expect("Unable to write to log file");
                writeln!(
                    log_file,
                    "   → [NOT-LD] Total blocked positions: {}",
                    sorted_positions.len()
                )
                .expect("Unable to write to log file");
            }
        }

        // If no blocked positions at all
        if algo_essential_blocked
            .get(file_name)
            .map_or(true, |s| s.is_empty())
            && algo_not_ld_blocked
                .get(file_name)
                .map_or(true, |s| s.is_empty())
        {
            writeln!(log_file, "   → No blocked positions").expect("Unable to write to log file");
        }

        writeln!(log_file, "").expect("Unable to write to log file");
    }

    // Add visual representation of blocked positions WITHOUT WALLS
    writeln!(log_file, "\nVISUAL REPRESENTATION OF BLOCKED POSITIONS")
        .expect("Unable to write to log file");
    writeln!(log_file, "===========================================")
        .expect("Unable to write to log file");
    writeln!(log_file, "").expect("Unable to write to log file");

    // Collect all unique blocked position indices across all algorithms
    let mut all_blocked_positions: std::collections::HashSet<usize> =
        std::collections::HashSet::new();
    for summary in summaries.iter() {
        all_blocked_positions.insert(summary.blocked_initial_positions_index);
    }

    // Create a map from position index to representative config
    let mut position_to_config: std::collections::HashMap<usize, &GridExperiment> =
        std::collections::HashMap::new();

    for exp in grid_experiments.iter() {
        if !position_to_config.contains_key(&exp.init_config.current_position_index) {
            position_to_config.insert(exp.init_config.current_position_index, &exp);
        }
    }

    // Sort position indices for consistent output
    let mut sorted_blocked_positions: Vec<usize> = all_blocked_positions.iter().cloned().collect();
    sorted_blocked_positions.sort();

    // Print visual representation for each blocked position
    for position_index in sorted_blocked_positions {
        if let Some(config) = position_to_config.get(&position_index) {
            writeln!(
                log_file,
                "-----------Position {}------------",
                position_index
            )
            .expect("Unable to write to log file");

            // Calculate inner grid dimensions (without walls)
            let inner_width = (config.grid_config.max_x - config.grid_config.min_x - 1) as usize;
            let inner_height = (config.grid_config.max_y - config.grid_config.min_y - 1) as usize;

            let mut grid = vec![vec!['.'; inner_width]; inner_height];

            // Place initial positions (only robots, no walls)
            for &(ch, x, y) in &config.init_config.initial_positions {
                // Convert to inner grid coordinates (excluding walls)
                let gx = (x - config.grid_config.min_x - 1) as usize;
                let gy = (config.grid_config.max_y - y - 1) as usize;
                if gy < inner_height && gx < inner_width {
                    grid[gy][gx] = ch;
                }
            }

            // Print the grid without walls
            for row in grid.iter() {
                write!(log_file, " ").expect("Unable to write to log file");
                for &cell in row {
                    write!(log_file, "{} ", cell).expect("Unable to write to log file");
                }
                writeln!(log_file).expect("Unable to write to log file");
            }

            // Show position details
            let positions_str: String = config
                .init_config
                .initial_positions
                .iter()
                .map(|(ch, x, y)| format!("('{}', {}, {})", ch, x, y))
                .collect::<Vec<_>>()
                .join(" ");

            writeln!(log_file, "Initial Positions: {}", positions_str)
                .expect("Unable to write to log file");
            writeln!(log_file, "").expect("Unable to write to log file");
        }
    }

    println!("Blocked summaries log written to '{}'", log_path.display());
}

fn create_inverse_blocked_summaries_log(
    base_path: &str,
    blocked_summaries: &Arc<Mutex<Vec<BlockedConfigSummary>>>,
    validation_configs: &[(ValidationConfig, usize)],
) {
    let log_path = Path::new(base_path).join("_inverse_blocked_summaries.log");
    let mut log_file =
        File::create(&log_path).expect("Unable to create inverse blocked summaries log file");

    writeln!(log_file, "FAILED ALGORITHMS BY POSITION").expect("Unable to write to log file");
    writeln!(log_file, "===============================").expect("Unable to write to log file");
    writeln!(log_file, "").expect("Unable to write to log file");

    let summaries = blocked_summaries.lock().unwrap();

    // Group by POSITION index and collect unique algorithm names
    let mut position_failed_algos: std::collections::HashMap<
        usize,
        std::collections::HashSet<String>,
    > = std::collections::HashMap::new();

    for summary in summaries.iter() {
        let entry = position_failed_algos
            .entry(summary.blocked_initial_positions_index)
            .or_insert_with(std::collections::HashSet::new);
        entry.insert(summary.algorithm_name.clone());
    }

    // Get all unique position indices from validation configs
    let mut position_configs: std::collections::HashMap<usize, Vec<(usize, &ValidationConfig)>> =
        std::collections::HashMap::new();

    for (config_index, (config, position_index)) in validation_configs.iter().enumerate() {
        let entry = position_configs
            .entry(*position_index)
            .or_insert_with(Vec::new);
        entry.push((config_index, config));
    }

    // Write results for each unique position index
    let mut sorted_position_indices: Vec<usize> = position_configs.keys().cloned().collect();
    sorted_position_indices.sort();

    for position_index in sorted_position_indices {
        if let Some(configs_for_position) = position_configs.get(&position_index) {
            // Use the first config as representative (they should have same initial positions)
            let (first_config_index, first_config) = configs_for_position[0];

            writeln!(
                log_file,
                "Position {} (appears in {} configs):",
                position_index,
                configs_for_position.len()
            )
            .expect("Unable to write to log file");

            // Show the initial positions for this position index
            let positions_str: String = first_config
                .initial_positions
                .iter()
                .map(|(ch, x, y)| format!("('{}', {}, {})", ch, x, y))
                .collect::<Vec<_>>()
                .join(" ");

            writeln!(log_file, "   → Initial Positions: {}", positions_str)
                .expect("Unable to write to log file");
            writeln!(log_file, "   → Example Config: {}", first_config_index)
                .expect("Unable to write to log file");

            if let Some(failed_algos) = position_failed_algos.get(&position_index) {
                if failed_algos.is_empty() {
                    writeln!(log_file, "   → No algorithms failed")
                        .expect("Unable to write to log file");
                } else {
                    let mut sorted_algos: Vec<String> = failed_algos.iter().cloned().collect();
                    sorted_algos.sort();
                    writeln!(log_file, "   → Failed algorithms:")
                        .expect("Unable to write to log file");
                    for algo in sorted_algos.iter() {
                        writeln!(log_file, "     • {}", algo).expect("Unable to write to log file");
                    }
                    writeln!(
                        log_file,
                        "   → Total failed algorithms: {}",
                        sorted_algos.len()
                    )
                    .expect("Unable to write to log file");
                }
            } else {
                writeln!(log_file, "   → No algorithms failed")
                    .expect("Unable to write to log file");
            }
            writeln!(log_file, "").expect("Unable to write to log file");
        }
    }

    println!(
        "Inverse blocked summaries log written to '{}'",
        log_path.display()
    );
}


pub fn write_algorithm_summary_log(
    directory_path: &str,
    results: &Arc<Mutex<Vec<String>>>,
    snapshot: &AlgorithmSnapshot,
    total_count: usize,
) -> io::Result<()> {
    let log_path = Path::new(directory_path).join("_validation_results.log");
    let mut log_file = File::create(&log_path).expect("Unable to create log file");

    let validated_ld_count = snapshot.validated_ld.len();
    let validated_not_ld_count = snapshot.validated_not_ld.len();
    let blocked_count = snapshot.blocked.len();
    let cyclic_count = snapshot.cyclic.len();
    let timeout_count = snapshot.timeout.len();
    writeln!(log_file, "VALIDATION SUMMARY (Algorithm-Level Status)")?;
    writeln!(
        log_file,
        "— Validated (LD):     {}/{}",
        validated_ld_count, total_count
    )?;
    write_names(&mut log_file, "      ↳", &snapshot.validated_ld)?;
    writeln!(
        log_file,
        "— Validated (NOT-LD): {}/{}",
        validated_not_ld_count, total_count
    )?;
    write_names(&mut log_file, "      ↳", &snapshot.validated_not_ld)?;

    writeln!(
        log_file,
        "— Blocked:            {}/{}",
        blocked_count, total_count
    )?;
    write_names(&mut log_file, "      ↳", &snapshot.blocked)?;

    writeln!(
        log_file,
        "— Cyclic:             {}/{}",
        cyclic_count, total_count
    )?;
    write_names(&mut log_file, "      ↳", &snapshot.cyclic)?;

    writeln!(
        log_file,
        "— Timeout:            {}/{}",
        timeout_count, total_count
    )?;
    write_names(&mut log_file, "      ↳", &snapshot.timeout)?;

    writeln!(log_file)?;

    let results = results.lock().map_err(|_| {
        io::Error::new(
            io::ErrorKind::Other,
            "Failed to acquire lock on results vector",
        )
    })?;

    for result in results.iter() {
        writeln!(log_file, "{}", result)?;
    }

    Ok(())
}

fn write_names(writer: &mut File, prefix: &str, entries: &[String]) -> io::Result<()> {
    if entries.is_empty() {
        return Ok(());
    }
    let joined = entries.join(", ");
    writeln!(writer, "{} {}", prefix, joined)
}

pub fn write_validation_summary_log(base_path: &str, checked_folders: &[(i16, String, bool)]) {
    let log_path = Path::new(base_path).join("_validation_summary.log");
    let mut log_file = match File::create(&log_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to create validation summary log: {}", e);
            return;
        }
    };

    writeln!(log_file, "VALIDATION SUMMARY").expect("Unable to write to log file");
    writeln!(log_file, "==================").expect("Unable to write to log file");
    writeln!(log_file, "").expect("Unable to write to log file");
    writeln!(log_file, "Checked folders in order:").expect("Unable to write to log file");
    writeln!(log_file, "").expect("Unable to write to log file");

    for (activation, folder_path, found_result) in checked_folders {
        let status = if *found_result {
            "[VALIDATED]"
        } else {
            "[NO RESULT]"
        };
        writeln!(log_file, "{} Activation {}", status, activation)
            .expect("Unable to write to log file");
        writeln!(log_file, "   Path: {}", folder_path).expect("Unable to write to log file");

        if *found_result {
            writeln!(
                log_file,
                "   Result: Found validated algorithms - stopped here"
            )
            .expect("Unable to write to log file");
        } else {
            writeln!(
                log_file,
                "   Result: No validated algorithms - moved to next level"
            )
            .expect("Unable to write to log file");
        }
        writeln!(log_file, "").expect("Unable to write to log file");
    }

    let final_folder = checked_folders.last();
    if let Some((activation, _, found_result)) = final_folder {
        writeln!(log_file, "Final result:").expect("Unable to write to log file");
        if *found_result {
            writeln!(
                log_file,
                "Validation stopped at activation {} with positive results",
                activation
            )
            .expect("Unable to write to log file");
        } else {
            writeln!(
                log_file,
                "All folders checked - no validated algorithms found"
            )
            .expect("Unable to write to log file");
        }
    }

    println!("\nValidation summary written to '{}'", log_path.display());
}
