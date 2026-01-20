use std::error::Error;
use std::fs::File;
use std::io::Read;

use crate::modules::structs::algorithm::Algorithm;
use crate::modules::structs::full_rule::print_full_rule;

pub fn load_algorithms() -> Result<Vec<Algorithm>, Box<dyn Error>> {
    let filename = concat!(env!("CARGO_MANIFEST_DIR"), "/src/data/algorithms_data.json");

    let mut file = File::open(filename)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;

    let algorithms: Vec<Algorithm> = serde_json::from_str(&data)?;
    Ok(algorithms)
}

pub fn simple_load() {
    match load_algorithms() {
        Ok(algorithms) => {
            println!("Loaded {} algorithms successfully!", algorithms.len());
            for algo in algorithms {
                println!("Algorithm Name: {}", algo.name);
                for (i, rule) in algo.rules.iter().enumerate() {
                    println!("---------------------{i}----------------------");
                    print_full_rule(rule);
                }
            }
        }
        Err(e) => eprintln!("Error loading algorithms: {}", e),
    }
}
