use crate::modules::{
    combination_mode::CombinationMode,
    generation_mode::GenerationMode,
    grid_config::GridConfig,
    init_config::InitConfig,
    simulation_config::{SimulationConfig, Target},
    validation_config::ValidationConfig,
};
use serde::{Deserialize, Serialize};

/// Comprehensive configuration struct that collects all configurations used in the project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // --- Basic Algorithm Parameters ---
    pub obstacle: char,
    pub number_of_robots: usize,
    pub number_of_colors: usize,
    pub visibility_range: i16,
    pub all_color_letters: Vec<char>,

    // --- Algorithm Generation and Processing ---
    pub existing_algorithm_path: String,
    pub generation_mode: GenerationMode,
    pub opacity: bool,
    pub is_obstacle_opaque: bool,

    // --- Obstacle and Visibility Settings ---

    // --- Web Algorithm Configuration ---
    pub web_algo_colors: String,
    pub web_algo_initial_configuration: String,
    pub web_algo_walls: Vec<[usize; 2]>,

    // --- Validation Configuration ---
    pub leader_colors: Vec<char>,
    pub moving_on_space_pattern: Vec<Vec<(char, i16, i16)>>,
    pub goals: Vec<SimulationConfig>,
    pub initial_configurations: Vec<(Vec<(char, i16, i16)>, bool)>,
}

//cretafn display_config(config: &Config) {
impl Config {
    pub fn display(&self) {
        println!(
            "╔══════════════════════════════════════════════════════════════════════════════╗"
        );
        println!("║                           CONFIGURATION DISPLAY                           ║");
        println!(
            "╚══════════════════════════════════════════════════════════════════════════════╝"
        );

        println!("\n📋 BASIC ALGORITHM PARAMETERS");
        println!("   Obstacle Character: {}", self.obstacle);
        println!("   Number of Robots: {}", self.number_of_robots);
        println!("   Number of Colors: {}", self.number_of_colors);
        println!("   Visibility Range: {}", self.visibility_range);
        println!("   All Color Letters: {:?}", self.all_color_letters);

        println!("\n🔧 ALGORITHM GENERATION AND PROCESSING");
        println!(
            "   Existing Algorithm Path: {}",
            self.existing_algorithm_path
        );
        println!("   Generation Mode: {:?}", self.generation_mode);
        println!("   Opacity: {}", self.opacity);
        println!("   Is Obstacle Opaque: {}", self.is_obstacle_opaque);

        println!("\n🌐 WEB ALGORITHM CONFIGURATION");
        println!("   Web Algo Colors: {}", self.web_algo_colors);
        println!(
            "   Web Algo Initial Configuration: {}",
            self.web_algo_initial_configuration
        );
        println!("   Web Algo Walls: {:?}", self.web_algo_walls);

        println!("\n✅ VALIDATION CONFIGURATION");
        println!("   Leader Colors: {:?}", self.leader_colors);
        println!(
            "   Moving on Space Pattern: {} patterns",
            self.moving_on_space_pattern.len()
        );
        for (i, pattern) in self.moving_on_space_pattern.iter().enumerate() {
            println!("     Pattern {}: {:?}", i + 1, pattern);
        }
        println!("   Goals: {} simulation configs", self.goals.len());
        for (i, goal) in self.goals.iter().enumerate() {
            println!("     Goal {}: {:?}", i + 1, goal);
        }
        println!(
            "   Initial Configurations: {} configs",
            self.initial_configurations.len()
        );
        for (i, (config, flag)) in self.initial_configurations.iter().enumerate() {
            println!("     Config {}: {:?} -> {}", i + 1, config, flag);
        }

        println!(
            "\n═══════════════════════════════════════════════════════════════════════════════"
        );
    }
}
