use std::path;
use std::process::Output;
use std::{collections::HashMap, fs};

use crate::classification::classification_generator::export_classification;
use crate::classification::comparison_generator::generate_multi_algorithm_viewer;
use crate::methodology::configuration::CONFIG;
use crate::modules::algorithm_experiments_modules::algorithm_experiments::AlgorithmExperiments;
use crate::modules::algorithm_status::AlgorithmStatus;

pub fn classify(experiments_path: &str, output_path: &str) {
    //let experiments_path = "src/to_classify";
    let mut experiments = get_experiment_files(experiments_path);
    println!("Found {} experiment files to classify.", experiments.len());

    //here iw ant to check that th experiments is not empty
    if experiments.is_empty() {
        eprintln!("No experiments to classify after filtering. Exiting.");
        return;
    }

    /*  let show_family_signatures = false; // Set to false to hide long signatures

        // ===== GLOBAL METRICS (Families 1-3) =====
        print_family(
            &experiments,
            1,
            "Rule Counts",
            show_family_signatures,
            |a| a.hash_by_rules_count(),
        );
        print_family(
            &experiments,
            2,
            "Idle Rule Counts",
            show_family_signatures,
            |a| a.hash_by_idle_rules_count(),
        );
        print_family(
            &experiments,
            3,
            "Opacity Rule Counts",
            show_family_signatures,
            |a| a.hash_by_opac_rules_count(),
        );

        // ===== BY ROBOT COLOR (Families 4-6) =====
        print_family(
            &experiments,
            4,
            "Rule Counts by Colors",
            show_family_signatures,
            |a| a.hash_family_by_rules_count_by_colors(),
        );
        print_family(
            &experiments,
            5,
            "Idle Rule Counts by Colors",
            show_family_signatures,
            |a| a.hash_family_by_idle_rules_count_by_colors(),
        );
        print_family(
            &experiments,
            6,
            "Opacity Rule Counts by Colors",
            show_family_signatures,
            |a| a.hash_family_by_opacity_rules_count_by_colors(),
        );

        // ===== EXECUTION PATTERNS - TOTAL (Families 7-11) =====
        print_family(
            &experiments,
            7,
            "Rules Count in Executions",
            show_family_signatures,
            |a| a.hash_by_rules_count_in_executions(),
        );
        print_family(
            &experiments,
            8,
            "Total Activation in Executions",
            show_family_signatures,
            |a| a.hash_by_total_activation_in_executions(),
        );
        print_family(
            &experiments,
            9,
            "Color Activation in Executions",
            show_family_signatures,
            |a| a.hash_by_color_activation_in_executions(),
        );
        print_family(
            &experiments,
            10,
            "Movement Activation in Executions",
            show_family_signatures,
            |a| a.hash_by_movement_activation_in_executions(),
        );
        print_family(
            &experiments,
            11,
            "Total Steps Taken in Executions",
            show_family_signatures,
            |a| a.hash_by_steps_taken_in_executions(),
        );
        print_family(
            &experiments,
            12,
            "Cycle Length in Executions",
            show_family_signatures,
            |a| a.hash_by_cycle_len_in_executions(),
        );

        // ===== EXECUTION PATTERNS - BY ROBOT (Families 13-16) =====
        print_family(
            &experiments,
            13,
            "Rules Count in Executions by Robot",
            show_family_signatures,
            |a| a.hash_by_rules_count_in_executions_by_robot(),
        );
        print_family(
            &experiments,
            14,
            "Total Activation in Executions by Robot",
            show_family_signatures,
            |a| a.hash_by_activation_in_executions_by_robot(),
        );
        print_family(
            &experiments,
            15,
            "Color Activation in Executions by Robot",
            show_family_signatures,
            |a| a.hash_by_color_activation_in_executions_by_robot(),
        );
        print_family(
            &experiments,
            16,
            "Movement Activation in Executions by Robot",
            show_family_signatures,
            |a| a.hash_by_movement_activation_in_executions_by_robot(),
        );

        // ===== FINAL POSITIONS (Family 17) =====

        print_family(
            &experiments,
            17,
            "Robot Colors in Executions by Robot",
            show_family_signatures,
            |a| a.hash_by_color_in_executions(),
        );

        print_family(
            &experiments,
            18,
            "Paths by Robot",
            show_family_signatures,
            |a| a.hash_by_paths_in_executions(),
        );

        print_family(
            &experiments,
            19,
            "Robot Positions in Executions by Robot",
            show_family_signatures,
            |a| a.hash_by_positions_in_executions(),
        );
    */
    // Export classification results to JSON
    export_classification(&experiments, output_path, "");
    // generate_comparison_html(&experiments, output_path, CONFIG.number_of_robots,"");
    generate_multi_algorithm_viewer(&experiments, output_path, "");
}

fn print_family<F>(
    experiments: &[AlgorithmExperiments],
    family_num: usize,
    title: &str,
    show_signature: bool,
    hash_fn: F,
) where
    F: Fn(&AlgorithmExperiments) -> String,
{
    println!("\n┌──────────────────────────────────────────────────────────────┐");
    println!("│  Family {:<2}: {:<48} │", family_num, title);
    println!("└──────────────────────────────────────────────────────────────┘");

    let families = group_by_hash(experiments, |a| hash_fn(a));

    for (hash, group) in families {
        if show_signature {
            println!("Family {} → {} algos", hash, group.len());
        } else {
            println!("Family → {} algos", group.len());
        }
        for algo in group {
            println!(" - {}", algo.name);
        }
    }
}

fn print_family_<F>(
    experiments: &[AlgorithmExperiments],
    family_num: usize,
    title: &str,
    hash_fn: F,
) where
    F: Fn(&AlgorithmExperiments) -> String,
{
    // Header box
    println!("\n╔═════════════════════════════════════════════════════════════════╗");
    println!("║  Family {:<2}: {:<48} ║", family_num, title);
    println!("╚═════════════════════════════════════════════════════════════════╝");

    // Group algorithms by hash
    let families: HashMap<String, Vec<&AlgorithmExperiments>> =
        group_by_hash(experiments, |a| hash_fn(a));

    // Sort families by hash for consistent order
    let mut sorted_keys: Vec<_> = families.keys().collect();
    sorted_keys.sort();

    // Print each family
    for key in sorted_keys {
        let group = &families[key];
        println!(
            "\n▶ Family [{}] — {} algorithm{}",
            key,
            group.len(),
            if group.len() > 1 { "s" } else { "" }
        );

        for algo in group {
            println!("   • {}", algo.name);
        }
    }

    // Optional summary
    println!(
        "\nTotal families: {} | Total algorithms: {}\n",
        families.len(),
        experiments.len()
    );
}

pub fn get_experiment_files(path: &str) -> Vec<AlgorithmExperiments> {
    let mut experiments = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    match serde_json::from_str::<AlgorithmExperiments>(&content) {
                        Ok(experiment_data) => {
                            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                                experiments.push(experiment_data);
                            }
                        }
                        Err(err) => {
                            eprintln!("⚠️ Failed to parse {}: {}", path.display(), err);
                        }
                    }
                }
            }
        }
    }

    experiments
}

pub fn group_by_hash<F>(
    algos: &[AlgorithmExperiments],
    hash_fn: F,
) -> HashMap<String, Vec<&AlgorithmExperiments>>
where
    F: Fn(&AlgorithmExperiments) -> String,
{
    let mut map: HashMap<String, Vec<&AlgorithmExperiments>> = HashMap::new();
    for algo in algos {
        let key = hash_fn(algo);
        map.entry(key).or_default().push(algo);
    }
    map
}
