use crate::classification::classification_generator::export_classification;
use crate::classification::comparison_generator::generate_multi_algorithm_viewer;
use crate::classification::logic::{classify, get_experiment_files};

use crate::methodology::configuration::COMBINATION_MODE;
use crate::methodology::globals::{
    are_in_same_opacity_group, get_all_color_letters, get_execution_root_str, get_number_of_colors,
    get_opacity_group_id, get_opacity_group_lookup, get_original_rules_count, get_parallel_rules,
};
use crate::methodology::goal_positions_viewer::generate_goal_positions_viewer;
use crate::methodology::goal_target_result::GoalTargetResult;
use crate::methodology::goals_viewer::generate_goals_viewer;
use crate::methodology::parallel_rules::{self, calculate_activation_counts};
use crate::methodology::switching_colors_validator::remove_duplicates_by_color_switches;
use crate::modules::algorithm;
use crate::modules::algorithm_experiments_modules::algorithm_experiments::AlgorithmExperiments;
use crate::modules::color::get_colors;
use crate::modules::combination_mode::CombinationMode;
use crate::modules::direction::calculate_movement;
use crate::modules::final_rule::FinalRule;
use crate::modules::generation_mode::GenerationMode;
use crate::modules::parallel_rules::{
    calculate_color_activation, calculate_total_activation, check_parallel_rules_compatibility,
    extract_rules,
};
use crate::modules::progress_helper::ProgressHelper;
use crate::modules::rule::Rule;

use crate::modules::time_helper::format_elapsed_time;
use crate::validation::initial_config_generator::generate_initial_configs;
use crate::validation::logic::{validate, validate_single_folder};
use crate::{
    methodology::globals::{get_rules, get_views},
    modules::{
        color,
        direction::Direction,
        execution_logger::log_note,
        folder_generator::FolderGenerator,
        parallel_rules::ParallelRules,
        rule,
        simulator::simulation,
        view::{are_equivalent_with_rotation, View},
        web_algo_generator::WebAlgoGenerator,
        yaml_algo_generator::YamlAlgoGenerator,
    },
};
use chrono::{format, Local};
use core::time;
use fxhash::FxHasher;
use itertools::{iproduct, Group, Unique};
use rayon::{prelude::*, range, vec};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::hash::{self, Hash, Hasher};
use std::io::{BufWriter, Write};
use std::path::{self, Path};
use std::process::exit;
use std::sync::atomic::AtomicIsize;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::configuration::CONFIG;
use super::globals::get_visibility;
use std::thread;

#[derive(Debug)]
struct ZeroCombinationFound;

pub fn run_simulation() -> usize {
    let folder_path = FolderGenerator::create_folder(&get_execution_root_str(), "Goals");
    let parallel_rules = get_parallel_rules();

    let simulation_configs = CONFIG.goals.clone();
    let mut list_of_executions: Vec<Vec<Vec<usize>>> = vec![];

    let mut goals_targets_details: Vec<Vec<GoalTargetResult>> = vec![];
    let goals_start = Instant::now(); // Start timing
    for (i, config) in simulation_configs.iter().enumerate() {
        let goal_start = Instant::now(); // Start timing
        let (temp_list_executions, temp_list_positions) = simulation(
            i + 1,
            &config.initial_positions,
            &config.targets,
            &parallel_rules,
            config.boundary,
            &get_views(),
            &get_rules(),
            *get_visibility(),
            CONFIG.opacity,
        );
        let mut targets_details: Vec<GoalTargetResult> = vec![];
        for (j, (executions, positions)) in temp_list_executions
            .iter()
            .zip(temp_list_positions.iter())
            .enumerate()
        {
            if executions.is_empty() {
                panic!("❌ No executions found for goal {} with target {}!\n💡 Edit goal {} and try again", i + 1, j + 1, i + 1);
            }
            let execution_filename = format!("goal_{}_target_{}_executions.json", i + 1, j + 1);
            let position_filename = format!("goal_{}_target_{}_positions.json", i + 1, j + 1);

            let filename = generate_goal_positions_viewer(
                &positions,
                &executions,
                folder_path.as_str(),
                i + 1,
                j + 1,
            );
            list_of_executions.push(executions.clone());
            targets_details.push(GoalTargetResult {
                execution_count: executions.len(),
                result_path: filename,
            });

            // j      -> index
            // executions -> element from list_executions
            // position   -> element from list_positions
            log_note(&format!(
                "Simulated goal {} with target {} in {} sec: found {} execution paths",
                i + 1,
                j + 1,
                format_elapsed_time(goal_start),
                executions.len()
            ));
        }

        goals_targets_details.push(targets_details);
    }

    log_note(&format!(
        "All goals simulated in {} sec",
        format_elapsed_time(goals_start)
    ));
    generate_goals_viewer(&goals_targets_details);

    let mut validated_global_algos: Vec<Vec<usize>> = Vec::new();

    match CONFIG.generation_mode {
        GenerationMode::All => {
            println!();
            // Mode 1: Generate all possible algorithms without filtering
            println!("🔄 Generation Mode: ALL - Generating all possible algorithms");
            validated_global_algos = combine(&list_of_executions, &parallel_rules);
            let validated_global_algos: Vec<Vec<usize>> =
                sort_validated_algorithms(validated_global_algos);
            println!("{} global algos sorted", validated_global_algos.len());

            // 2. Remove duplicate rules within each algorithm
            let (filtered_algos, original_rules_indices_cleaned) =
                convert_and_deduplicate_rules_in_each_algorithm(
                    &validated_global_algos,
                    CONFIG.opacity,
                );

            let (unique_algos, removed_indices, hashed, runs) = if CONFIG.opacity {
                remove_duplicates_indexed_simple_fast(filtered_algos)
            } else {
                (filtered_algos, Vec::new(), Vec::new(), Vec::new())
            };

            // 3. Remove duplicate algorithms by permutation of color switches
            let cleaned_algos =
                remove_duplicates_by_color_switches(unique_algos, &original_rules_indices_cleaned);

            // 6. Generate output files
            let global_folder = FolderGenerator::create_folder(&get_execution_root_str(), "Algos");
            generate_all_algorithms_files(
                &cleaned_algos,
                &global_folder,
                CONFIG.opacity,
                &original_rules_indices_cleaned,
            );
            if hashed.len() > 0 {
                log_hash_stats(&hashed, &runs, &cleaned_algos, &global_folder);
            }
            // Validate single folder
            let validation_start = Instant::now();
            validate_single_folder(&global_folder);
            log_note(&format!(
                "Validation completed in {}",
                format_elapsed_time(validation_start)
            ));

            // Classification
            let classification_start = Instant::now();
            classify(
                format!("{}/Algos", &get_execution_root_str()).as_str(),
                global_folder.as_str(),
            );

            log_note(&format!(
                "Classification completed in {}",
                format_elapsed_time(classification_start)
            ));

            cleaned_algos.len()
        }

        GenerationMode::ProgressiveValidationByLevels(max_levels) => {
            let prograsive_start_time = Instant::now();
            //generate all possible algorithms without filtering
            println!();
            println!(
                "🔄 Generation Mode: ProgressiveValidation (max {} levels) ",
                max_levels
            );
            validated_global_algos = combine(&list_of_executions, &parallel_rules);

            //calculate activations for all algorithms
            let list_of_activations = validated_global_algos
                .iter()
                .map(|algo| calculate_total_activation(algo, &parallel_rules))
                .collect::<Vec<usize>>();

            //Group algorithm indices by activation level
            let mut activation_groups: HashMap<usize, Vec<usize>> = HashMap::new();
            let mut max_activation = 0;

            for (algo_idx, activation) in list_of_activations.iter().enumerate() {
                activation_groups
                    .entry(*activation)
                    .or_default()
                    .push(algo_idx);
            }
            //Sort each group's algorithm indices
            for indices in activation_groups.values_mut() {
                indices.sort_unstable();
            }

            //Convert to sorted vector of (activation, indices)
            let mut result: Vec<(usize, Vec<usize>)> = activation_groups.into_iter().collect();

            max_activation = result
                .last()
                .map(|(k, _)| *k)
                .unwrap_or(0)
                .to_string()
                .len();

            // Sort by activation level (ascending)
            result.sort_unstable_by_key(|(activation, _)| *activation);

            log_note(&format!(
                "{} levels found, activation levels: {:?}",
                result.len(),
                result.iter().map(|(k, _)| *k).collect::<Vec<usize>>()
            ));

            println!("\n╔═══════════════════════════════════════════════════════╗");
            println!("║  PROGRESSIVE VALIDATION - Activation Level Analysis  ║");
            println!(
                "║  (Processing max {} levels)                            ║",
                max_levels
            );
            println!("╚═══════════════════════════════════════════════════════╝");

            let mut algorithm_index = 0;
            let mut list_of_validation_results: Vec<String> = vec![];
            for (i, (activation_level, algorithm_set)) in result.iter().enumerate() {
                let _start_time = Instant::now();
                println!("\n┌─────────────────────────────────────────────────────┐");
                println!(
                    "│ 📊 Activation Level: {} ({} algorithms)",
                    activation_level,
                    algorithm_set.len()
                );
                println!("└─────────────────────────────────────────────────────┘");

                //reoder algorithms by their original indices to ensure consistent processing
                let mut ordered_indices = algorithm_set.clone(); // copy the indices
                ordered_indices
                    .sort_by(|&a, &b| validated_global_algos[a].cmp(&validated_global_algos[b]));

                //

                let mut algos: Vec<Vec<usize>> = ordered_indices
                    .iter()
                    .map(|&idx| validated_global_algos[idx].clone())
                    .collect();

                let (filtered_algos, original_rules_indices_cleaned) =
                    convert_and_deduplicate_rules_in_each_algorithm(&algos, CONFIG.opacity);

                let mut hashed: Vec<(u64, usize, AlgorithmSignature)> = Vec::new();
                let mut runs: Vec<std::ops::Range<usize>> = Vec::new();
                if CONFIG.opacity {
                    let (unique_algos, _, hashed_, runs_) =
                        remove_duplicates_indexed_simple_fast(filtered_algos);
                    algos = unique_algos;
                    hashed = hashed_;
                    runs = runs_;
                } else {
                    algos = filtered_algos;
                };

                // 3. Remove duplicate algorithms by permutation of color switches
                let cleaned_algos =
                    remove_duplicates_by_color_switches(algos, &original_rules_indices_cleaned);

                let count = cleaned_algos.len();

                // Format: 080_contains_2
                let folder_name = format!(
                    "{:0width$}_contains_{}",
                    activation_level,
                    count,
                    width = max_activation
                );

                let folder_path = format!("{}/Algos/{}", &get_execution_root_str(), folder_name);

                list_of_validation_results.push(folder_path.clone());
                // Create directory
                if let Err(e) = fs::create_dir_all(&folder_path) {
                    eprintln!("⚠️ Failed to create directory {}: {}", folder_path, e);
                    continue;
                }

                println!("📁 Folder: {}", folder_name);
                println!("🔧 Generating {} algorithm files...", count);

                // Generate algorithm files in this directory

                for (index, algo) in cleaned_algos.iter().enumerate() {
                    algorithm_index += 1;
                    let output_name = format!("algo_{}_act_{}", algorithm_index, activation_level);
                    generate_web_algo(
                        &algo,
                        &folder_path,
                        &output_name,
                        &get_views(),
                        &original_rules_indices_cleaned,
                    );
                }
                if hashed.len() > 0 {
                    log_hash_stats(&hashed, &runs, &cleaned_algos, &folder_path);
                }

                println!("✅ Files generated");
                // Validate single folder
                //try to add in th lognote the number of algorithms validated
                let algorithm_snapshot = validate_single_folder(folder_path.as_str());
                log_note(&format!(
                    "Activation level {} processed in {} : {} algorithms validated",
                    activation_level,
                    format_elapsed_time(_start_time),
                    algorithm_snapshot.validated_ld.len()
                ));

                if max_levels > 0 && i + 1 >= max_levels {
                    println!(
                        "\n⏹️  Stopping after {} activation levels (configured limit)",
                        max_levels
                    );
                    break;
                }
            }

            generate_global_classification_report(
                &list_of_validation_results,
                &get_execution_root_str(),
            );

            println!("\n╔═══════════════════════════════════════════════════════╗");
            println!("║       PROGRESSIVE VALIDATION COMPLETED                ║");
            println!("╚═══════════════════════════════════════════════════════╝\n");

            log_note(&format!(
                "Progressive validation completed in {}",
                format_elapsed_time(prograsive_start_time)
            ));
            0
        }
    }
}

pub fn generate_global_classification_report(
    list_of_validation_results: &Vec<String>,
    output_path: &str,
) {
    println!("\n📊 Global Classification Report...");
    let mut experiments: Vec<AlgorithmExperiments> = vec![];
    for folder in list_of_validation_results {
        let path = format!("{}/_details/", folder);
        println!("Validating folder: {}", path);
        let new_experiments = get_experiment_files(&path);
        experiments.extend(new_experiments);
    }
    println!("Algorithms collected (Validated): {}", experiments.len());
    if experiments.is_empty() {
        eprintln!("No experiments to classify after filtering. Exiting.");
        return;
    }
    println!("file:{}/", output_path);
    export_classification(&experiments, format!("{}", output_path).as_str(), "global_");
    generate_multi_algorithm_viewer(&experiments, format!("{}", output_path).as_str(), "global_");
}

/// Log hash collision statistics to file
fn log_hash_stats(
    hashed: &[(u64, usize, AlgorithmSignature)],
    runs: &[std::ops::Range<usize>],
    algorithms: &[Vec<usize>],
    base_folder: &str,
) {
    let path: String = format!("{}/_hash_stats.txt", base_folder);
    let file = match File::create(&path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("⚠️  Failed to create hash stats file: {}", e);
            return;
        }
    };
    let mut writer = BufWriter::new(file);

    // Header
    let _ = writeln!(
        writer,
        "═══════════════════════════════════════════════════"
    );
    let _ = writeln!(writer, "         HASH COLLISION STATISTICS");
    let _ = writeln!(
        writer,
        "═══════════════════════════════════════════════════"
    );
    let _ = writeln!(writer, "Total Algorithms: {}", algorithms.len());
    let _ = writeln!(writer, "Unique Hashes:    {}", runs.len());
    let _ = writeln!(
        writer,
        "═══════════════════════════════════════════════════\n"
    );

    // Each hash bucket
    for (i, range) in runs.iter().enumerate() {
        let hash = hashed[range.start].0;
        let sig = &hashed[range.start].2;
        let count = range.len();

        let _ = writeln!(
            writer,
            "Hash #{}: {:016x} → {} algorithm(s)",
            i + 1,
            hash,
            count
        );

        // Show signature (same for all in bucket, so just pick first)
        let _ = writeln!(
            writer,
            "  Rules/color:  {:?}",
            sig.rule_count_by_robot_color
        );
        let _ = writeln!(
            writer,
            "  Activation:   {:?}",
            sig.color_activation_count_by_robot_color
        );
        let _ = writeln!(writer, "  Movement:     {}", sig.movement_activation_count);
        let _ = writeln!(
            writer,
            "  Idle/color:   {:?}",
            sig.idle_rule_count_by_robot_color
        );
        let _ = writeln!(
            writer,
            "  Opacity:      {:?} groups\n",
            sig.opacity_groups_sorted
        );
    }

    let _ = writeln!(
        writer,
        "Generated: {}",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    );
    println!("📊 Hash statistics saved to: {}", path);
}

fn verify_compatibility(rule_id_1: &usize, rule_id_2: &usize) -> bool {
    if rule_id_1 == rule_id_2 {
        return true; // Items are the same, so they are compatible
    }

    if get_rules()[*rule_id_1].view_id == get_rules()[*rule_id_2].view_id
        && (get_rules()[*rule_id_1].direction != get_rules()[*rule_id_2].direction
            || get_rules()[*rule_id_1].color != get_rules()[*rule_id_2].color)
    {
        return false; // Return false if compatibility check fails
    }

    true // Return true if all checks pass
}
fn distribute_executions_v2(
    global_goals: &HashSet<Vec<usize>>,
    parallel_rules_goals: &[Vec<usize>],
    list_of_parallel_rules: &[ParallelRules],
) -> HashSet<Vec<usize>> {
    use rayon::prelude::*; // rayon for parallel iteration

    let global_set = if global_goals.is_empty() {
        parallel_rules_goals
            .par_iter()
            .map(|goal| goal.clone())
            .collect::<HashSet<_>>()
    } else {
        parallel_rules_goals
            .par_iter()
            .flat_map_iter(|parallel_rules_goal| {
                global_goals.iter().map(move |global_goal| {
                    let mut local_set = HashSet::new();
                    process_global_goals(
                        global_goal,
                        parallel_rules_goal,
                        &mut local_set,
                        list_of_parallel_rules,
                    );
                    local_set
                })
            })
            .flatten()
            .collect::<HashSet<_>>()
    };

    global_set
}
pub fn distribute_executions_v2c(
    global_executions: &Vec<Vec<usize>>,
    current_executions: &[Vec<usize>],
    list_of_parallel_rules: &[ParallelRules],
) -> Vec<Vec<usize>> {
    if global_executions.is_empty() {
        current_executions.par_iter().cloned().collect()
    } else {
        current_executions
            .par_iter()
            .flat_map(|current_execution| {
                global_executions
                    .par_iter()
                    .filter_map(move |global_execution| {
                        let mut local_set = merge_executions(
                            global_execution,
                            current_execution,
                            list_of_parallel_rules,
                            &get_rules(),
                            CONFIG.opacity,
                        );
                        if !local_set.is_empty() {
                            Some(local_set)
                        } else {
                            None
                        }
                    })
            })
            .collect()
    }
}

fn bi_combine_all(
    executions_list: &[Vec<Vec<usize>>],
    list_of_parallel_rules: &[ParallelRules],
) -> HashSet<Vec<usize>> {
    let mut current: Vec<HashSet<Vec<usize>>> = executions_list
        .iter()
        .enumerate()
        .map(|(i, exec)| {
            let set = HashSet::from_iter(exec.clone());
            println!("Initial exec {} -> {} elements", i + 1, set.len());
            set
        })
        .collect();

    let mut round = 1;
    while current.len() > 1 {
        println!("\n=== Round {} ===", round);

        let mut next = Vec::new();

        for (i, pair) in current.chunks(2).enumerate() {
            if pair.len() == 1 {
                println!(
                    " Pair {}: only one set ({} elems), carried forward",
                    i + 1,
                    pair[0].len()
                );
                next.push(pair[0].clone());
            } else {
                println!(
                    " Pair {}: combining sets of size {} and {}",
                    i + 1,
                    pair[0].len(),
                    pair[1].len()
                );

                let mut combined = HashSet::new();
                for g in &pair[0] {
                    for h in &pair[1] {
                        let merged_goal = merge_executions(
                            g,
                            h,
                            list_of_parallel_rules,
                            &get_rules(),
                            CONFIG.opacity,
                        );

                        if !merged_goal.is_empty() {
                            combined.insert(merged_goal);
                        }
                        //  process_global_goals(g, h, &mut combined, list_of_parallel_rules);
                    }
                }
                if combined.is_empty() {
                    return HashSet::new();
                }
                println!("  -> Result has {} elements", combined.len());
                next.push(combined);
            }
        }

        println!("End of round {} -> {} sets remaining", round, next.len());
        current = next;
        round += 1;
    }

    println!(
        "\nFinal result has {} elements",
        current.first().map(|s| s.len()).unwrap_or(0)
    );

    current.into_iter().next().unwrap_or_default()
}

fn parallel_combine_all(
    executions_list: &[Vec<Vec<usize>>],
    list_of_parallel_rules: &[ParallelRules],
) -> HashSet<Vec<usize>> {
    println!(
        "Starting parallel combination of {} execution lists",
        executions_list.len()
    );

    // Convert each execution list to a HashSet for faster lookups
    let mut current: Vec<HashSet<Vec<usize>>> = executions_list
        .iter()
        .enumerate()
        .map(|(i, exec)| {
            let set = HashSet::from_iter(exec.clone());
            println!("Initial exec {} -> {} elements", i + 1, set.len());
            set
        })
        .collect();

    let mut round = 1;
    while current.len() > 1 {
        println!("\n=== Parallel Round {} ===", round);

        // Process pairs in parallel using rayon
        let result: Result<Vec<HashSet<Vec<usize>>>, ZeroCombinationFound> = current
            .par_chunks(2)
            .enumerate()
            .map(|(i, pair)| {
                if pair.len() == 1 {
                    println!(
                        " Pair {}: only one set ({} elems), carried forward",
                        i + 1,
                        pair[0].len()
                    );
                    Ok(pair[0].clone()) // ✅ wrapped in Ok(..)
                } else {
                    println!(
                        " Pair {}: combining sets of size {} and {} in parallel",
                        i + 1,
                        pair[0].len(),
                        pair[1].len()
                    );

                    let combined: HashSet<Vec<usize>> = iproduct!(pair[0].iter(), pair[1].iter())
                        .par_bridge()
                        .filter_map(|(g, h)| {
                            let merged_goal = merge_executions(
                                g,
                                h,
                                list_of_parallel_rules,
                                &get_rules(),
                                CONFIG.opacity,
                            );
                            (!merged_goal.is_empty()).then_some(merged_goal)
                        })
                        .collect();

                    if combined.is_empty() {
                        Err(ZeroCombinationFound) // 🚨 short-circuit
                    } else {
                        println!("  -> Parallel result has {} elements", combined.len());
                        Ok(combined)
                    }
                }
            })
            .collect();

        match result {
            Ok(next) => {
                println!(
                    "End of parallel round {} -> {} sets remaining",
                    round,
                    next.len()
                );
                current = next;
                round += 1;
            }
            Err(_) => {
                println!("🚨 Empty combination detected -> returning {{0}} immediately");
                return HashSet::new();
            }
        }
    }

    let final_result = current.into_iter().next().unwrap_or_default();
    println!(
        "\nParallel combination final result has {} elements",
        final_result.len()
    );

    final_result
}

fn is_compatible(
    global_goal: &[usize],
    parallel_rules_goal: &[usize],
    list_of_parallel_rules: &[ParallelRules],
) -> bool {
    for &parallel_rule_a in parallel_rules_goal {
        for &parallel_rule_b in global_goal {
            // Ensure that indices are within bounds of list_of_parallel_rules
            if parallel_rule_a >= list_of_parallel_rules.len()
                || parallel_rule_b >= list_of_parallel_rules.len()
            {
                return false;
            }

            // Compare rules within the parallel rule a and b
            for (rule_a, _, _, _, _) in &list_of_parallel_rules[parallel_rule_a].rules {
                for (rule_b, _, _, _, _) in &list_of_parallel_rules[parallel_rule_b].rules {
                    if !verify_compatibility(rule_a, rule_b) {
                        return false;
                    }
                }
            }
        }
    }
    true
}

fn extract_list_of_idle_robots_views(
    parallel_rules_goal: &[usize],
    list_of_parallel_rules: &[ParallelRules],
) -> Vec<View> {
    let mut list_of_idle_robots_views = Vec::new();

    for &parallel_rule_index in parallel_rules_goal {
        list_of_idle_robots_views.extend(
            list_of_parallel_rules[parallel_rule_index]
                .idle_robots_views
                .iter()
                .cloned(),
        );
    }

    list_of_idle_robots_views
}

fn process_global_goals(
    global_goal: &Vec<usize>,      // A previously validated combination of rules
    parallel_rules_goal: &[usize], // An execution (set of parallel rule indices) from the current goal
    newly_formed_valid_global_set: &mut HashSet<Vec<usize>>, // Accumulator for valid merged goals in this step
    list_of_parallel_rules: &[ParallelRules],
) {
    // Check compatibility between the existing global_goal and the current parallel_rules_goal
    let merged_goal = merge_executions(
        global_goal,
        parallel_rules_goal,
        list_of_parallel_rules,
        &get_rules(),
        CONFIG.opacity,
    );

    if !merged_goal.is_empty() {
        newly_formed_valid_global_set.insert(merged_goal);
    }
}

fn process_global_goalss(
    global_goal: &Vec<usize>,      // A previously validated combination of rules
    parallel_rules_goal: &[usize], // An execution (set of parallel rule indices) from the current goal
    newly_formed_valid_global_set: &mut HashSet<Vec<usize>>, // Accumulator for valid merged goals in this step
    list_of_parallel_rules: &[ParallelRules],
) {
    // Check compatibility between the existing global_goal and the current parallel_rules_goal
    let merged_execution = merge_executions(
        global_goal,
        parallel_rules_goal,
        list_of_parallel_rules,
        &get_rules(),
        CONFIG.opacity,
    );

    newly_formed_valid_global_set.insert(merged_execution);
}

fn is_contain_forbidden_view(
    merged_goal: &[usize],
    idle_robots_views: &[View],
    list_of_parallel_rules: &[ParallelRules],
) -> bool {
    for &parallel_rule_id in merged_goal.iter() {
        for (rule, _, _, _, _) in list_of_parallel_rules[parallel_rule_id].rules.iter() {
            let view_id = get_rules()[*rule].view_id;
            for idle_robot_view in idle_robots_views.iter() {
                if are_equivalent_with_rotation(&get_views()[view_id], &idle_robot_view) {
                    return true;
                }
            }
        }
    }
    false
}
fn are_executions_consistent(
    execution_a: &[usize],
    execution_b: &[usize],
    list_of_parallel_rules: &[ParallelRules],
) -> bool {
    for &parallel_rule_a in execution_a {
        for &parallel_rule_b in execution_b {
            // Ensure that indices are within bounds of list_of_parallel_rules
            if parallel_rule_a >= list_of_parallel_rules.len()
                || parallel_rule_b >= list_of_parallel_rules.len()
            {
                return false;
            }

            // Compare rules within the parallel rule a and b
            for (rule_a, _, _, _, _) in &list_of_parallel_rules[parallel_rule_a].rules {
                for (rule_b, _, _, _, _) in &list_of_parallel_rules[parallel_rule_b].rules {
                    if !verify_compatibility(rule_a, rule_b) {
                        return false;
                    }
                }
            }
        }
    }
    true
}

/// Merge two executions (vectors of parallel rule indices) into one
fn merge_executions(
    execution_a: &[usize],
    execution_b: &[usize],
    list_of_parallel_rules: &[ParallelRules],
    rules: &[rule::Rule],
    opacity: bool,
) -> Vec<usize> {
    // if the two executions are not compatible, return empty vec
    if !are_executions_compatible(
        execution_a,
        execution_b,
        list_of_parallel_rules,
        rules,
        opacity,
    ) {
        return Vec::new();
    }

    let mut merged = execution_a.to_vec();
    for &idx in execution_b {
        if !merged.contains(&idx) {
            merged.push(idx);
        }
    }
    merged

    //test if this idle views are compatible with
}

pub fn are_executions_compatible(
    exec_a: &[usize],
    exec_b: &[usize],
    list_of_parallel_rules: &[ParallelRules],
    rules: &[Rule],
    opacity: bool,
) -> bool {
    // Check every pair of parallel rules between the two executions
    for &idx_a in exec_a {
        for &idx_b in exec_b {
            // Skip if it's the same parallel rule (can't conflict with itself)
            if idx_a == idx_b {
                continue;
            }

            if !check_parallel_rules_compatibility(
                &list_of_parallel_rules[idx_a],
                &list_of_parallel_rules[idx_b],
                rules,
                opacity,
                &get_views(),
                &get_visibility(),
            ) {
                return false;
            }
        }
    }
    true
}

fn ganerate_yaml_algo(
    algorithm: &[usize],
    dir: &str,
    filename: &str,
    list_of_parallel_rules: &[ParallelRules],
) {
    let template = r#"algorithm:
  initial_configuration:
{initial_configurations}
{rules}
{footer}
"#;

    let mut generator = YamlAlgoGenerator::new(filename, dir, template);

    generator.set(
        "initial_configurations",
        r#"    grid7x6:
    - [W, W, W, W, W, W, W, W, W, W]
    - [W, ., ., ., ., ., ., ., ., W]
    - [W, ., B, G, ., ., ., ., ., W]
    - [W, ., ., ., ., O, ., ., ., W]
    - [W, ., ., ., ., ., ., ., ., W]
    - [W, ., ., ., ., ., ., ., ., W]
    - [W, ., ., ., ., ., ., ., ., W]
    - [W, ., ., ., ., ., ., ., ., W]
    - [W, W, W, W, W, W, W, W, W, W]
    

  alias:
    X: "{.,W}"
  rules:
  "#,
    );

    generator.add_rule(r#"# Existing rules"#);
    let existed_rules_len = get_original_rules_count();

    for i in 0..existed_rules_len {
        let rule = &get_rules()[i];

        let rule_string = generate_yaml_string_rule(
            &get_views()[rule.view_id],
            rule.direction.clone(),
            rule.color,
            *get_visibility(),
        );
        generator.add_rule(rule_string.as_str());
    }
    generator.add_rule(r#"# New rules!"#);

    let new_rules = extract_rules(algorithm, list_of_parallel_rules);
    for rule_id in new_rules {
        if rule_id > existed_rules_len - 1 {
            // Add the rule only if it hasn't been added before
            generator.add_rule(&format!(r#"# Rule n: {}"#, rule_id));
            let rule = &get_rules()[rule_id];
            let rule_string = generate_yaml_string_rule(
                &get_views()[rule.view_id],
                rule.direction.clone(),
                rule.color,
                *get_visibility(),
            );
            generator.add_rule(rule_string.as_str());
        }
    }
    generator.set(
        "footer",
        "


  grid_size: 20
  n_round: 300
  offset: [8, 8]
graphics:
  colors:
    W: black
    B: blue
    R: red
    G: green
    O: orange
model:
  chirality: true",
    );

    generator.save_yaml();
}

fn generate_web_algo(
    new_rules: &[usize],
    dir: &str,
    filename: &str,
    views: &[View],
    original_rules_indices_cleaned: &[usize],
) {
    let template = r"****** OPTIONS ******
version: 1
walls:
  - - {wall_x0}
    - {wall_y0}
    - 0
  - - {wall_x1}
    - {wall_y1}
    - 2
chirality: true
visibilityRange: {visibility}
colors:
{colors}
dimension: 2

****** INITIAL CONFIGURATIONS ******
{initial_configurations}

****** RULES ******
@alias X {alias_x}

{rules}
";

    let mut generator = WebAlgoGenerator::new(&filename, dir, template);
    generator.set("visibility", &get_visibility().to_string());
    // Set wall positions from constants
    generator.set("wall_x0", &CONFIG.web_algo_walls[0][0].to_string());
    generator.set("wall_y0", &CONFIG.web_algo_walls[0][1].to_string());
    generator.set("wall_x1", &CONFIG.web_algo_walls[1][0].to_string());
    generator.set("wall_y1", &CONFIG.web_algo_walls[1][1].to_string());
    generator.set("colors", &CONFIG.web_algo_colors);
    generator.set(
        "initial_configurations",
        &CONFIG.web_algo_initial_configuration,
    );
    generator.set("alias_x", &build_alias_x());

    for &rule_id in new_rules {
        generator.add_rule(&format!("# New rule: {}", rule_id));

        let rule = &get_rules()[rule_id];
        let rule_string = generate_web_algo_string_rule(
            &views[rule.view_id],
            rule.direction.clone(),
            rule.color,
            *get_visibility(),
        );
        generator.add_rule(rule_string.as_str());
        // }
    }

    for &rule_id in original_rules_indices_cleaned {
        generator.add_rule(&format!("# Existing rule: {}", rule_id));

        let rule = &get_rules()[rule_id];
        let rule_string = generate_web_algo_string_rule(
            &views[rule.view_id],
            rule.direction.clone(),
            rule.color,
            *get_visibility(),
        );
        generator.add_rule(rule_string.as_str());
        // }
    }
    generator.save_web_algo();
}

fn generate_yaml_string_rule(
    points: &[(char, i16, i16)],
    dir: Direction,
    color: char,
    visibility: i16,
) -> String {
    let len = (visibility * 2 + 1) as usize;

    // Initialize grid
    let mut grid = vec![vec![' '; len]; len];

    // Mark visible area
    for j in -visibility..=visibility {
        for i in -visibility..=visibility {
            if i.abs() + j.abs() <= visibility {
                grid[(j + visibility) as usize][(i + visibility) as usize] = '.';
            }
        }
    }

    // Place characters in the grid
    for &(ch, x, y) in points {
        if x.abs() + y.abs() > visibility {
            panic!("this position is outside of bounds");
        }
        let center = visibility;
        let new_x = center as isize + x as isize;
        let new_y = center as isize - y as isize;

        grid[new_y as usize][new_x as usize] = ch;
    }

    // Convert grid to YAML format
    let mut rule_string = String::new();
    rule_string.push_str("  - \"");

    for (i, row) in grid.iter().enumerate() {
        if i > 0 {
            rule_string.push_str("      "); // Add indentation only for rows after the first one
        }
        rule_string.push_str(&row.iter().collect::<String>());
        rule_string.push_str("\n");
    }

    rule_string.pop(); // Remove last newline
    rule_string.push_str("\"\n");

    if dir == Direction::Up {
        rule_string.push_str(&format!("\n  - (up, {})\n", color));
    } else if dir == Direction::Down {
        rule_string.push_str(&format!("  - (down, {})\n", color));
    } else if dir == Direction::Left {
        rule_string.push_str(&format!("  - (left, {})\n", color));
    } else if dir == Direction::Right {
        rule_string.push_str(&format!("  - (right, {})\n", color));
    } else {
        rule_string.push_str(&format!("  - (idle, {})\n", color));
    }

    rule_string
}

fn generate_web_algo_string_rule(
    points: &[(char, i16, i16)],
    dir: Direction,
    color: char,
    visibility: i16,
) -> String {
    let len = (visibility * 2 + 1) as usize;

    // Initialize grid
    let mut grid = vec![vec![' '; len]; len];

    // Mark visible area
    for j in -visibility..=visibility {
        for i in -visibility..=visibility {
            if i.abs() + j.abs() <= visibility {
                grid[(j + visibility) as usize][(i + visibility) as usize] = '.';
            }
        }
    }

    // Place characters in the grid
    for &(ch, x, y) in points {
        if x.abs() + y.abs() > visibility {
            panic!("this position is outside of bounds");
        }
        let center = visibility;
        let new_x = center as isize + x as isize;
        let new_y = center as isize - y as isize;

        grid[new_y as usize][new_x as usize] = ch;
    }

    // Convert grid to YAML format
    let mut rule_string = String::new();
    rule_string.push_str(" \n");

    for (i, row) in grid.iter().enumerate() {
        let row_str: String = row.iter().collect();
        rule_string.push_str(&row_str);

        // Append movement rule at the end of the visibility index row
        if i == visibility as usize {
            let dir_str = match dir {
                Direction::Up => "front",
                Direction::Down => "back",
                Direction::Left => "left",
                Direction::Right => "right",
                Direction::Idle => "idle",
            };
            rule_string.push_str(&format!(" -> {}, {}", dir_str, color));
        }
        rule_string.push('\n');
    }

    rule_string
}

pub fn calculate_activation_levels(
    list_of_activation: &Vec<Vec<usize>>,
) -> (Vec<Vec<usize>>, usize) {
    let mut max_level = 0;

    let levels: Vec<Vec<usize>> = list_of_activation
        .iter()
        .map(|vals| {
            let mut seen = HashSet::new();
            let mut inner = Vec::new();
            for v in vals {
                if seen.insert(v) {
                    inner.push(*v);
                }
            }
            inner.sort_unstable();
            if inner.len() > max_level {
                max_level = inner.len();
            }
            inner
        })
        .collect();

    (levels, max_level)
}

pub fn filter_by_levels(
    list_of_executions: &Vec<Vec<Vec<usize>>>,
    list_of_activations: &Vec<Vec<usize>>,
    list_of_levels: &Vec<Vec<usize>>,
    max_level: usize,
) -> Vec<Vec<Vec<usize>>> {
    let mut filtered = Vec::new();

    for ((executions, activations), levels) in list_of_executions
        .iter()
        .zip(list_of_activations.iter())
        .zip(list_of_levels.iter())
    {
        // Determine which levels to keep
        let keep_levels: Vec<usize> = levels.iter().take(max_level).cloned().collect();

        // Find indices of activations belonging to kept levels
        let mut indices = Vec::new();
        for (idx, act) in activations.iter().enumerate() {
            if keep_levels.contains(act) {
                indices.push(idx);
            }
        }

        // Filter executions by indices
        let kept_execs: Vec<Vec<usize>> = indices
            .into_iter()
            .filter_map(|i| executions.get(i).cloned())
            .collect();

        filtered.push(kept_execs);
    }

    filtered
}
fn combine(
    list_of_executions: &Vec<Vec<Vec<usize>>>,
    list_of_parallel_rules: &[ParallelRules],
) -> Vec<Vec<usize>> {
    let combination_start: Instant = Instant::now(); // Start timing
    let mut global_algos: Vec<Vec<usize>> = Vec::new();

    match COMBINATION_MODE {
        CombinationMode::Sequential => {
            for (i, executions) in list_of_executions.iter().enumerate() {
                global_algos =
                    distribute_executions_v2c(&global_algos, &executions, &list_of_parallel_rules);
                if global_algos.is_empty() {
                    panic!("❌ No valid combination after Goal {}!\n💡 Edit goal {} and try again", i + 1, i + 1);
                }
                println!(
                    "After goal {}: {} global algos found!",
                    i + 1,
                    global_algos.len()
                );

                log_note(&format!(
                    "After goal {} in {}: {} global algos found!",
                    i + 1,
                    format_elapsed_time(combination_start),
                    global_algos.len()
                ));
            }
        }
        CombinationMode::BiCombination => {
            global_algos = bi_combine_all(&list_of_executions, &list_of_parallel_rules)
                .into_iter()
                .collect();
        }
        CombinationMode::Parallel => {
            global_algos = parallel_combine_all(&list_of_executions, &list_of_parallel_rules)
                .into_iter()
                .collect();
        }
    }

    log_note(&format!(
        "combinaison in {}",
        format_elapsed_time(combination_start),
    ));
    // manager.write_executions(&global_algos, "global_algos.json");
    global_algos
}

fn sort_validated_algorithms(validated_global_algos: Vec<Vec<usize>>) -> Vec<Vec<usize>> {
    // Convert HashSet to Vec for sorting
    let mut validated_global_algos_vec: Vec<Vec<usize>> =
        validated_global_algos.into_iter().collect();

    // Sort each individual algorithm (inner Vec)
    for algo in validated_global_algos_vec.iter_mut() {
        algo.sort();
    }

    // Sort the outer Vec for deterministic order
    validated_global_algos_vec.sort();
    validated_global_algos_vec
}

/// Convert all algorithms from parallel rules to rule-based format
fn convert_algorithms_to_final_rules(validated_global_algos: &[Vec<usize>]) -> Vec<Vec<FinalRule>> {
    println!(
        "🔄 Converting {} algorithms from parallel rules to rule-based format...",
        validated_global_algos.len()
    );

    validated_global_algos
        .iter()
        .enumerate()
        .map(|(algo_index, algorithm)| {
            let rules = convert_single_algorithm_to_final_rules(algorithm);
            println!(
                "  Algorithm {}: {} parallel rules → {} final rules",
                algo_index + 1,
                algorithm.len(),
                rules.len()
            );
            rules
        })
        .collect()
}

/// Convert a single algorithm (list of parallel rule indices) to final rules
fn convert_single_algorithm_to_final_rules(algorithm: &[usize]) -> Vec<FinalRule> {
    let mut final_rules = Vec::new();
    let mut unique_rules = std::collections::HashSet::new();

    // Extract all unique rule IDs from the parallel rules
    for &parallel_rule_index in algorithm {
        for &(rule_id, _, _, _, _) in &get_parallel_rules()[parallel_rule_index].rules {
            unique_rules.insert(rule_id);
        }
    }

    // Convert each unique rule to final rule format
    for rule_id in unique_rules {
        let rule = &get_rules()[rule_id];
        let view = &get_views()[rule.view_id];

        final_rules.push(FinalRule {
            view: view.clone(),
            direction: rule.direction,
            color: rule.color,
        });
    }

    final_rules
}

/// Detect blocked configurations for all rule-based algorithms
fn detect_blocked_configurations(
    algorithms_with_rules: &[Vec<FinalRule>],
    max_steps: usize,
) -> Vec<Vec<usize>> {
    println!(
        "🔍 Testing {} rule-based algorithms for blocked configurations...",
        algorithms_with_rules.len()
    );

    let initial_positions_configs = generate_initial_configs(
        CONFIG
            .moving_on_space_pattern
            .iter()
            .map(|v| {
                v.iter()
                    .map(|&(c, x, y)| (c, x as i16, y as i16))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        CONFIG.leader_colors.to_vec(),
        *get_visibility(),
        CONFIG.opacity,
    );
    let mut blocked_results: Vec<Vec<usize>> = vec![Vec::new(); algorithms_with_rules.len()];

    for (algo_index, algorithm_rules) in algorithms_with_rules.iter().enumerate() {
        println!(
            "  Testing algorithm {} with {} rules against {} position configs...",
            algo_index + 1,
            algorithm_rules.len(),
            initial_positions_configs.len()
        );

        for (config_index, initial_positions) in initial_positions_configs.iter().enumerate() {
            if is_configuration_blocked(&initial_positions.0, algorithm_rules, max_steps) {
                blocked_results[algo_index].push(config_index);
                println!("    → Config {} is BLOCKED ❌", config_index);
            } else {
                println!("    → Config {} is OK ✅", config_index);
            }
        }

        if blocked_results[algo_index].is_empty() {
            println!(
                "  ✅ Algorithm {} has NO blocked configurations",
                algo_index + 1
            );
        } else {
            println!(
                "  ❌ Algorithm {} has {} blocked configurations: {:?}",
                algo_index + 1,
                blocked_results[algo_index].len(),
                blocked_results[algo_index]
            );
        }
    }

    blocked_results
}

/// Simulate a configuration for several steps to detect blocking
fn is_configuration_blocked(
    initial_positions: &[(char, i16, i16)],
    algorithm_rules: &[FinalRule], // Changed parameter name for consistency
    max_steps: usize,
) -> bool {
    let mut current_positions = initial_positions.to_vec();
    let mut previous_positions = Vec::new();

    for step in 0..max_steps {
        let mut next_positions = Vec::new();
        let mut any_robot_moved = false;

        // Apply rules for each robot
        for (robot_index, &robot) in current_positions.iter().enumerate() {
            let robot_view = create_robot_view(robot, &current_positions, robot_index);

            if let Some((direction, color)) = find_matching_rule(&robot_view, algorithm_rules) {
                let (new_x, new_y) = calculate_movement(&direction, &robot.1, &robot.2);
                next_positions.push((color, new_x, new_y));

                if new_x != robot.1 || new_y != robot.2 {
                    any_robot_moved = true;
                }
            } else {
                // No matching rule - robot stays in place
                next_positions.push(robot);
            }
        }

        // Check for blocking conditions
        if !any_robot_moved {
            return true; // No robot moved - configuration is blocked
        }

        // Check for cycles (robot positions repeat)
        if step > 0 && next_positions == previous_positions {
            return true; // Position cycle detected - configuration is blocked
        }

        previous_positions = current_positions.clone();
        current_positions = next_positions;
    }

    false // Completed all steps without blocking
}

/// Create robot's view of surrounding robots (no walls, just other robots)
fn create_robot_view(
    robot: (char, i16, i16),
    all_positions: &[(char, i16, i16)],
    robot_index: usize,
) -> Vec<(char, i16, i16)> {
    let (robot_char, robot_x, robot_y) = robot;
    let mut view = vec![(robot_char, 0, 0)]; // Robot sees itself at center (0,0)

    let visibility = *get_visibility();

    // Add other robots within visibility range
    for (i, &(other_char, other_x, other_y)) in all_positions.iter().enumerate() {
        if i != robot_index {
            let relative_x = other_x - robot_x;
            let relative_y = other_y - robot_y;

            // Check Manhattan distance
            if relative_x.abs() + relative_y.abs() <= visibility {
                view.push((other_char, relative_x, relative_y));
            }
        }
    }

    view
}
/// Find matching rule for robot view with rotation support
fn find_matching_rule(
    robot_view: &[(char, i16, i16)],
    algorithm_rules: &[FinalRule], // Changed parameter name for consistency
) -> Option<(Direction, char)> {
    use crate::modules::direction::rotate_direction;
    use crate::modules::view::are_equivalent;
    use crate::modules::view::rotate_view;

    let rotation_angles = [0, 90, 180, 270];

    // Get robot character from center of view
    let robot_char = robot_view[0].0;

    // Filter rules that could match (same robot character at center)
    let candidate_rules: Vec<&FinalRule> = algorithm_rules
        .iter()
        .filter(|rule| !rule.view.is_empty() && rule.view[0].0 == robot_char)
        .collect();

    for rule in candidate_rules {
        for &angle in &rotation_angles {
            let rotated_rule_view = rotate_view(&rule.view, angle);
            let rotated_direction = rotate_direction(&rule.direction, angle);

            if are_equivalent(&robot_view.to_vec(), &rotated_rule_view) {
                return Some((rotated_direction, rule.color));
            }
        }
    }

    None
}

pub fn remove_duplicate_rules(
    mut rules_of_algorithm: Vec<usize>,
    rules: &[Rule],
    opacity: bool,
) -> Vec<usize> {
    let mut i = 0;
    while i < rules_of_algorithm.len() {
        let rule_a_idx = rules_of_algorithm[i];
        let mut j = i + 1;
        while j < rules_of_algorithm.len() {
            let rule_b_idx = rules_of_algorithm[j];
            let same_view = rules[rule_a_idx].view_id == rules[rule_b_idx].view_id;
            let same_opacity_group = opacity
                && are_in_same_opacity_group(rules[rule_a_idx].view_id, rules[rule_b_idx].view_id);

            if same_view || same_opacity_group {
                // remove rule_b (do not increment j, since elements shifted left)
                rules_of_algorithm.remove(j);
            } else {
                j += 1;
            }
        }
        i += 1;
    }
    rules_of_algorithm
}

pub fn remove_duplicate_rules_in_algorithm(
    mut rules_of_algorithm: Vec<usize>,
    original_base_rules: &[usize],
    rules: &[Rule],
    opacity: bool,
) -> Vec<usize> {
    let mut i = 0;
    'outer: while i < rules_of_algorithm.len() {
        let rule_a_idx = rules_of_algorithm[i];
        let mut j = i + 1;
        while j < rules_of_algorithm.len() {
            let rule_b_idx = rules_of_algorithm[j];
            let same_view = rules[rule_a_idx].view_id == rules[rule_b_idx].view_id;
            let same_opacity_group = opacity
                && are_in_same_opacity_group(rules[rule_a_idx].view_id, rules[rule_b_idx].view_id);

            if same_view || same_opacity_group {
                // remove rule_b (do not increment j, since elements shifted left)
                rules_of_algorithm.remove(j);
            } else {
                j += 1;
            }
        }
        let mut k = 0;
        while k < original_base_rules.len() {
            let rule_b_idx = original_base_rules[k];
            let same_view = rules[rule_a_idx].view_id == rules[rule_b_idx].view_id;
            let same_opacity_group = opacity
                && are_in_same_opacity_group(rules[rule_a_idx].view_id, rules[rule_b_idx].view_id);

            if same_view || same_opacity_group {
                // remove rule_b (do not increment j, since elements shifted left)
                rules_of_algorithm.remove(i);

                //jump to next i
                continue 'outer;
            } else {
                k += 1;
            }
        }

        i += 1;
    }
    rules_of_algorithm
}

fn build_alias_x() -> String {
    let mut alias = vec!['.', 'W', 'O']; // fixed ones
    let num_colors = get_number_of_colors();

    // take only as many as needed from ALL_COLOR_LETTERS
    alias.extend_from_slice(
        &CONFIG.all_color_letters[..num_colors.min(CONFIG.all_color_letters.len())],
    );

    // turn into string like "{.,W,O,F,L,G}"
    let joined: String = alias
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{}}}", joined)
}

fn convert_and_deduplicate_rules_in_each_algorithm(
    validated_algos: &[Vec<usize>],
    opacity: bool,
) -> (Vec<Vec<usize>>, Vec<usize>) {
    let progress = ProgressHelper::new(validated_algos.len() as u64, "Cleaning duplicate rules");

    let original_rules_indices: Vec<usize> = (0..get_original_rules_count()).collect();
    let original_rules_indices_cleaned =
        remove_duplicate_rules(original_rules_indices, &get_rules(), opacity);

    let result: Vec<Vec<usize>> = validated_algos
        .par_iter()
        .map(|algorithm| {
            let new_rule_indices = extract_rules(&algorithm.clone(), &get_parallel_rules());

            let cleaned_rules = remove_duplicate_rules_in_algorithm(
                new_rule_indices,
                &original_rules_indices_cleaned,
                &get_rules(),
                opacity,
            );

            progress.inc();
            cleaned_rules
        })
        .collect();

    progress.finish_success("Cleaning duplicate rules", validated_algos.len());
    (result, original_rules_indices_cleaned)
}

fn generate_all_algorithms_files(
    filtered_algos: &[Vec<usize>],
    global_folder: &str,
    opacity: bool,
    original_rules_indices_cleaned: &[usize],
) {
    let progress = ProgressHelper::new(filtered_algos.len() as u64, "Generating algorithm files");
    let digits = filtered_algos.len().to_string().len();

    for (i, rule_indices) in filtered_algos.iter().enumerate() {
        let index = format!("{:0width$}", i + 1, width = digits);
        let output_name = format!("algo_{}", index);

        generate_web_algo(
            rule_indices,
            global_folder,
            &output_name,
            &get_views(),
            original_rules_indices_cleaned,
        );

        progress.inc();
    }

    progress.finish_success("File generation", filtered_algos.len());
}

/// Check if two algorithms are equivalent based on rule indices and CONFIG.opacitygroups
fn are_algorithms_equivalent(algo_a: &[usize], algo_b: &[usize], rules: &[Rule]) -> bool {
    for &rule_a in algo_a {
        let mut match_found = false;

        for &rule_b in algo_b {
            // Check if both rules are the same or belong to the same CONFIG.opacitygroup
            if rule_a == rule_b
                || are_in_same_opacity_group(rules[rule_a].view_id, rules[rule_b].view_id)
            {
                match_found = true;
                break; // ✅ stop checking once we found a match
            }
        }

        // ✅ if no match found for this rule_a, algorithms are not equivalent
        if !match_found {
            return false;
        }
    }

    true
}

fn get_activation_lists(
    list_of_executions: &[Vec<Vec<usize>>],
    list_of_parallel_rules: &[ParallelRules],
) -> Vec<Vec<usize>> {
    list_of_executions
        .iter()
        .map(|execution| {
            execution
                .iter()
                .map(|ex| calculate_total_activation(&ex, list_of_parallel_rules))
                .collect()
        })
        .collect()
}

/// Print a timestamped log message
fn log_with_timestamp(message: &str) {
    let now = Local::now();
    println!("[{}] {}", now.format("%Y-%m-%d %H:%M:%S"), message);
}

/// Group algorithms by activation and generate organized directories
fn generate_algorithms_grouped_by_activation(
    unique_algos: &[Vec<usize>],
    activation_counts: &[usize],
    base_folder: &str,
    opacity: bool,
    original_rules_indices_cleaned: &[usize],
    hashed: &[(u64, usize, AlgorithmSignature)],
    runs: &[std::ops::Range<usize>],
) -> std::io::Result<()> {
    use std::fs;

    println!("📊 Grouping algorithms by activation...");

    // Step 1: Group algorithm indices by activation value
    let mut activation_groups: HashMap<usize, Vec<usize>> = HashMap::new();

    for (algo_idx, &activation) in activation_counts.iter().enumerate() {
        activation_groups
            .entry(activation)
            .or_default()
            .push(algo_idx);
    }

    // Step 2: Sort activation values
    let mut sorted_activations: Vec<usize> = activation_groups.keys().copied().collect();
    sorted_activations.sort_unstable();

    // Step 3: Find max activation value to determine padding width
    let max_activation = sorted_activations.last().copied().unwrap_or(0);
    let padding_width = max_activation.to_string().len();

    println!(
        "📊 Found {} unique activation levels",
        sorted_activations.len()
    );
    println!(
        "   Activation range: {} to {}",
        sorted_activations.first().unwrap_or(&0),
        max_activation
    );

    // Step 4: Create directories and generate files for each activation level
    for activation in sorted_activations {
        let algo_indices = &activation_groups[&activation];
        let count = algo_indices.len();

        // Format: 080_contains_2
        let folder_name = format!(
            "{:0width$}_contains_{}",
            activation,
            count,
            width = padding_width
        );

        let folder_path = format!("{}/Algos/{}", base_folder, folder_name);

        // Create directory
        fs::create_dir_all(&folder_path)?;

        println!("   📁 Created: {} ({} algorithms)", folder_name, count);

        // Generate algorithm files in this directory
        generate_algorithms_in_folder(
            unique_algos,
            algo_indices,
            &folder_path,
            opacity,
            original_rules_indices_cleaned,
        );
        if hashed.len() > 0 {
            log_hash_stats(&hashed, &runs, &unique_algos, &folder_path);
        }

        // Validate single folder
        validate_single_folder(&folder_path);
    }

    println!("✅ Algorithm grouping completed!");
    Ok(())
}

/// Generate algorithm files in a specific folder
fn generate_algorithms_in_folder(
    unique_algos: &[Vec<usize>],
    algo_indices: &[usize],
    folder_path: &str,
    opacity: bool,
    original_rules_indices_cleaned: &[usize],
) {
    let digits = algo_indices.len().to_string().len();

    for (local_idx, &global_idx) in algo_indices.iter().enumerate() {
        let index = format!("{:0width$}", local_idx + 1, width = digits);
        let output_name = format!("algo_{}", index);

        generate_web_algo(
            &unique_algos[global_idx],
            folder_path,
            &output_name,
            &get_views(),
            original_rules_indices_cleaned,
        );
    }
}

/// Simple & fast rotation-invariant dedup (O(n log n) sort + parallel runs)
/// Returns (unique_algorithms, removed_indices_in_input_order)
pub fn remove_duplicates_indexed_simple_fast(
    algorithms: Vec<Vec<usize>>,
) -> (
    Vec<Vec<usize>>,
    Vec<usize>,
    Vec<(u64, usize, AlgorithmSignature)>,
    Vec<std::ops::Range<usize>>,
) {
    let n = algorithms.len();
    if n <= 1 {
        return (algorithms, Vec::new(), Vec::new(), Vec::new());
    }

    let rules = get_rules();

    // 1) Compute 64-bit hash of the rotation-invariant signature in parallel.
    println!("🔄 Computing algorithm signatures...");
    let mut hashed: Vec<(u64, usize, AlgorithmSignature)> = algorithms
        .par_iter()
        .enumerate()
        .map(|(i, algo)| {
            let sig = build_signature_from_algorithm(algo);
            let mut h = FxHasher::default();
            sig.hash(&mut h);
            (h.finish(), i, sig)
        })
        .collect();

    // 2) Parallel sort by hash so equal keys are contiguous
    hashed.par_sort_unstable_by_key(|&(h, _, _)| h);

    // 3) Identify contiguous runs of equal hashes
    let mut runs = Vec::new();
    if !hashed.is_empty() {
        let mut start = 0usize;
        for i in 1..hashed.len() {
            if hashed[i].0 != hashed[start].0 {
                runs.push(start..i);
                start = i;
            }
        }
        runs.push(start..hashed.len());
    }

    // 📊 Log hash collision statistics
    //log_hash_stats(&hashed, &runs, &algorithms, list_of_parallel_rules);

    // 4) Process runs in parallel with progress
    let progress = ProgressHelper::new(runs.len() as u64, "Checking for duplicate algorithms");

    let dup_pairs: Vec<(usize, usize)> = runs
        .par_iter()
        .map(|range| {
            let mut leaders: Vec<usize> = Vec::new();
            let mut pairs: Vec<(usize, usize)> = Vec::new();
            for &(_, idx, _) in &hashed[range.clone()] {
                if let Some(&rep) = leaders.iter().find(|&&rep| {
                    are_algorithms_equivalent(&algorithms[idx], &algorithms[rep], &rules)
                }) {
                    pairs.push((idx, rep));
                } else {
                    leaders.push(idx);
                }
            }
            progress.inc();
            pairs
        })
        .flatten()
        .collect();
    progress.finish_success("Checking for duplicate algorithms", runs.len());

    // 5) Assemble outputs deterministically in input order
    let mut rep_of: Vec<Option<usize>> = vec![None; n];
    for (i, rep) in dup_pairs {
        rep_of[i] = Some(rep);
    }

    let mut unique = Vec::with_capacity(n);
    let mut removed_indices = Vec::new();
    for (i, algo) in algorithms.into_iter().enumerate() {
        if rep_of[i].is_some() {
            removed_indices.push(i);
        } else {
            unique.push(algo);
        }
    }

    (unique, removed_indices, hashed, runs)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AlgorithmSignature {
    rule_count_by_robot_color: Vec<usize>,
    color_activation_count_by_robot_color: Vec<usize>,
    movement_activation_count: usize,
    idle_rule_count_by_robot_color: Vec<usize>,
    opacity_groups_sorted: Vec<usize>,
}

/// Build rotation-invariant signature
fn build_signature_from_algorithm(algo: &[usize]) -> AlgorithmSignature {
    let rules = get_rules();
    let colors: Vec<char> = get_colors(&get_all_color_letters(), *get_number_of_colors());
    let rule_count_by_robot_color = get_rule_count_by_robot_colors(algo, &colors);
    let color_activation_count_by_robot_color =
        get_color_activation_count_by_robot_colors(algo, &colors);
    let movement_activation_count = calculate_movement_activation_for_algorithm(algo);

    let idle_rule_count_by_robot_color = idle_rule_count_by_robot_color(algo, &colors);

    // Collect CONFIG.opacitygroups using O(1) global lookup
    let mut opacity_groups: Vec<usize> = algo
        .iter()
        .filter_map(|&rule_idx| {
            let rule = &rules[rule_idx];
            get_opacity_group_id(rule.view_id)
        })
        .collect();

    opacity_groups.sort_unstable();

    AlgorithmSignature {
        rule_count_by_robot_color,
        color_activation_count_by_robot_color,
        movement_activation_count,
        idle_rule_count_by_robot_color,
        opacity_groups_sorted: opacity_groups,
    }
}

pub fn get_rule_count_by_robot_colors(algo: &[usize], colors: &[char]) -> Vec<usize> {
    let mut counts = vec![0; colors.len()];
    let rules = get_rules();
    let views = get_views();

    for &rule_idx in algo {
        let view_id = rules[rule_idx].view_id;
        let view = &views[view_id];

        // Check if the robot at center (0,0) has a color
        if !view.is_empty() {
            let color = view[0].0;
            if let Some(color_idx) = colors.iter().position(|&c| c == color) {
                counts[color_idx] += 1;
            }
        }
    }

    counts
}

pub fn get_color_activation_count_by_robot_colors(algo: &[usize], colors: &[char]) -> Vec<usize> {
    let mut counts = vec![0; colors.len()];
    let rules = get_rules();

    for &rule_idx in algo {
        let activation_color = rules[rule_idx].color;
        if let Some(color_idx) = colors.iter().position(|&c| c == activation_color) {
            counts[color_idx] += 1;
        }
    }

    counts
}

pub fn calculate_movement_activation_for_algorithm(algo: &[usize]) -> usize {
    let rules = get_rules();
    algo.iter()
        .filter(|&&rule_idx| rules[rule_idx].direction != Direction::Idle)
        .count()
}

pub fn idle_rule_count_by_robot_color(algo: &[usize], colors: &[char]) -> Vec<usize> {
    colors
        .iter()
        .map(|&color| {
            algo.iter()
                .filter(|&&rule_idx| {
                    let rule = &get_rules()[rule_idx];
                    rule.direction == Direction::Idle && rule.color == color
                })
                .count()
        })
        .collect()
}
