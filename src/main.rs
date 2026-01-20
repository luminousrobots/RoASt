#![allow(warnings)]

//#![allow(dead_code)]
//#![allow(unused_imports)]
//#![allow(unused_variables)]

mod classification;
mod methodology;
mod modules;
mod validation;
use classification::logic::classify;
use serde_json;
use std::{env, fs, path::PathBuf, process::exit};
use validation::logic::validate;

use crate::{
    methodology::{
        cache::{clean_all, load_all},
        globals::{get_execution_root_str, init_execution_root},
        logic::methodology,
        simulator::run_simulation,
    },
    modules::execution_logger::{end_logger, init_logger},
    validation::{
        initial_config_generator::generate_initial_configs,
        initial_config_viewer::initial_config_viewer_html, logic::validate_single_folder,
    },
};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.len() {
        // No arguments - default simulation
        0 => {
            init_execution_root();
            init_logger(&get_execution_root_str(), "generation");
            methodology();
            end_logger();
        }

        // Single argument commands
        1 => match args[0].as_str() {
            "--use-cache" => {
                if load_all() {
                    init_execution_root();

                    init_logger(&get_execution_root_str(), "generation");
                    run_simulation();
                    end_logger();
                }
            }

            "--clean-cache" => {
                clean_all();
            }

            "--validate-last" => {
                if let Some(last_folder) = get_last_execution_folder() {
                    println!(
                        "Validating last execution folder: {}",
                        last_folder.display()
                    );
                    init_logger(last_folder.to_str().unwrap(), "validation");
                    validate(last_folder.to_str().unwrap());
                    end_logger();
                } else {
                    println!("No execution folders found in results.");
                }
            }

            "--classify" => {
                init_logger("to_classify", "classification");
                classify("to_classify", "to_classify");
                end_logger();
            }
            "--validate" => {
                init_logger("to_validate", "validation");
                validate_single_folder("to_validate");
                end_logger();
            }

            _ => {
                println!("Unknown command: {}", args[0]);
                print_usage();
            }
        },

        // Two argument commands
        2 => {
            if args[0] == "--validate-option" {
                // Must not contain / - only base folder name
                if args[1].contains('/') {
                    println!("Error: --validate-option requires a base folder name without '/'");
                    println!("Example: Execution_2025-10-14_16-05-22");
                    println!("For specific folders, use --validate-direct");
                    return;
                }

                let (path, _) = resolve_validation_paths(&args[1]);
                if path.exists() && path.is_dir() {
                    println!("Validating folder with hierarchy check: {}", path.display());
                    init_logger(path.to_str().unwrap(), "validation");
                    validate(path.to_str().unwrap());
                    end_logger();
                } else {
                    println!("Folder does not exist: {}", path.display());
                }
            } else if args[0] == "--validate-direct" {
                let (execution_path, target_path) = resolve_validation_paths(&args[1]);

                if !target_path.exists() {
                    println!("Error: Folder does not exist: {}", target_path.display());
                    return;
                }

                if !target_path.is_dir() {
                    println!("Error: Path is not a directory: {}", target_path.display());
                    return;
                }

                // Check if folder contains .web-algo files
                let has_algo_files = fs::read_dir(&target_path)
                    .ok()
                    .map(|entries| {
                        entries
                            .filter_map(|e| e.ok())
                            .any(|e| e.path().extension().map_or(false, |ext| ext == "web-algo"))
                    })
                    .unwrap_or(false);

                if !has_algo_files {
                    println!(
                        "Error: No .web-algo files found in: {}",
                        target_path.display()
                    );
                    println!("Please specify a folder that contains algorithm files.");
                    return;
                }

                println!(
                    "Direct validation (no hierarchy): {}",
                    target_path.display()
                );
                println!("Using execution context: {}", execution_path.display());
                init_logger(target_path.to_str().unwrap(), "validation");
                validate_single_folder(target_path.to_str().unwrap());
                end_logger();
            } else {
                println!("Unknown command: {}", args.join(" "));
                print_usage();
            }
        }

        // Too many arguments
        _ => {
            println!("Too many arguments: {}", args.join(" "));
            print_usage();
        }
    }
}

fn resolve_validation_paths(input: &str) -> (PathBuf, PathBuf) {
    // Extract execution folder name from various path formats
    let execution_folder = extract_execution_folder(input);
    let execution_path = PathBuf::from(&execution_folder);

    // For execution path - apply the same logic as before
    let final_execution_path = if execution_path.is_absolute()
        || execution_folder.starts_with("./")
        || execution_folder.starts_with("../")
    {
        execution_path
    } else if execution_path.exists() {
        execution_path
    } else {
        let with_results_prefix = PathBuf::from("results").join(&execution_folder);
        if with_results_prefix.exists() {
            with_results_prefix
        } else {
            with_results_prefix
        }
    };

    // For target path - resolve the original input
    let target_path = PathBuf::from(input);
    let final_target_path =
        if target_path.is_absolute() || input.starts_with("./") || input.starts_with("../") {
            target_path
        } else if target_path.exists() {
            target_path
        } else {
            let with_results_prefix = PathBuf::from("results").join(input);
            if with_results_prefix.exists() {
                with_results_prefix
            } else {
                with_results_prefix
            }
        };

    (final_execution_path, final_target_path)
}

fn extract_execution_folder(input: &str) -> String {
    // Remove leading slash if present
    let cleaned_input = input.strip_prefix('/').unwrap_or(input);

    // Split by '/' and find the execution folder pattern
    let parts: Vec<&str> = cleaned_input.split('/').collect();

    for part in &parts {
        // Just check if this part starts with "Execution_"
        if part.starts_with("Execution_") {
            return part.to_string();
        }
    }

    // If no execution folder found, return the input as-is
    input.to_string()
}

fn print_usage() {
    println!("Usage:");
    println!("  cargo run                           # Default behavior");
    println!("  cargo run -- --use-cache            # Use cached data");
    println!("  cargo run -- --clean-cache          # Clean all cache files");
    println!("  cargo run -- --classify             # Run classification");
    println!("  cargo run -- --validate             # Run validation");
    println!("  cargo run -- --validate-last        # Validate last execution");
    println!("  cargo run -- --validate-option NAME # Validate with hierarchy check");
    println!("  cargo run -- --validate-direct PATH # Direct validation (skip hierarchy)");
    println!("  cargo run -- --classify             # Run classification");
    println!("");
    println!("Examples for --validate-option (with hierarchy):");
    println!("  cargo run --release -- --validate-option Execution_2025-10-14_16-05-22");
    println!("  cargo run --release -- --validate-option Execution_2025-10-14_16-05-22");
    println!("");
    println!("Examples for --validate-direct (no hierarchy, just validate folder):");
    println!("  cargo run --release -- --validate-direct Execution_2025-10-14_16-05-22/Algos/75_contains_1");
    println!("  cargo run --release -- --validate-direct Execution_2025-10-14_16-05-22/Algos/90_contains_2");
}

fn get_last_execution_folder() -> Option<PathBuf> {
    let results_path = PathBuf::from("results");
    if !results_path.exists() {
        return None;
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(&results_path)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();

    // Sort existing folder names (Execution_YYYY-MM-DD_HH-MM-SS) lexicographically
    entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    entries.pop() // return the last existing one
}
