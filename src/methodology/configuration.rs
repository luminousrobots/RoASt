use std::{fs, path::Path, process::exit};

use once_cell::sync::Lazy;

use crate::modules::{
    combination_mode::CombinationMode, config::Config, generation_mode::GenerationMode,
    grid_config, init_config, simulation_config::SimulationConfig, validation_config,
};
//pub static MAX_EXPLORATION_STEPS: usize = 2000;
pub static CONFIG: Lazy<Config> = Lazy::new(|| load_or_create_config());
pub static COMBINATION_MODE: CombinationMode = CombinationMode::Sequential;

fn load_or_create_config() -> Config {
    let path = "config.json";

    // If file exists → try reading it
    if Path::new(path).exists() {
        if let Ok(text) = fs::read_to_string(path) {
            if let Ok(cfg) = serde_json::from_str::<Config>(&text) {
                cfg.display();
                return cfg; // success
            } else {
                println!(
                    "❌ Failed to parse config.json (invalid JSON format), using default config."
                );
            }
        } else {
            println!("❌ Failed to read config.json (file access error), using default config.");
        }
    } else {
        println!("ℹ️ config.json not found");
    }

    exit(0);
    // If reading/parsing fails → create default
    return create_config_algo1();
}

pub fn create_config_algo1() -> Config {
    Config {
        obstacle: 'O',
        number_of_robots: 2,
        number_of_colors: 3,
        visibility_range: 1,
        all_color_letters: vec!['F', 'L', 'R'],
        existing_algorithm_path: "/src/data/algorithms_data1.json".to_string(),
        generation_mode: GenerationMode::All,
        opacity: true,
        is_obstacle_opaque: true,
        web_algo_colors: r#"  L: 16711680
  F: 255
  R: 32768
  O: 16753920"#
            .to_string(),

        web_algo_initial_configuration: r#"WWWWWWWWW
W.......W
W.......W
W.FR....W
W...O...W
W.......W
W.......W
W.......W
WWWWWWWWW"#
            .to_string(),
        web_algo_walls: vec![[0, 0], [9, 9]],
        leader_colors: vec!['L', 'R'],
        moving_on_space_pattern: vec![
            vec![('R', 0, 0), ('F', -1, 0)],
            vec![('L', 0, 0), ('F', -1, 0)],
        ],
        goals: get_gaols_algo1(),
        initial_configurations: vec![
            (vec![('O', 0, 0), ('R', 1, 1), ('F', 0, 1)], false),
            (vec![('O', 0, 0), ('R', 0, 1), ('F', -1, 1)], true),
            (vec![('O', 0, 0), ('R', 2, 0), ('F', 1, 0)], false),
            (vec![('O', 0, 0), ('R', -1, 0), ('F', -2, 0)], true),
            (vec![('O', 0, 0), ('R', 1, -1), ('F', 0, -1)], false),
            (vec![('O', 0, 0), ('R', 0, -1), ('F', -1, -1)], true),
            (vec![('O', 0, 0), ('L', 1, 1), ('F', 0, 1)], false),
            (vec![('O', 0, 0), ('L', 0, 1), ('F', -1, 1)], true),
            (vec![('O', 0, 0), ('L', 2, 0), ('F', 1, 0)], false),
            (vec![('O', 0, 0), ('L', -1, 0), ('F', -2, 0)], true),
            (vec![('O', 0, 0), ('L', 1, -1), ('F', 0, -1)], false),
            (vec![('O', 0, 0), ('L', 0, -1), ('F', -1, -1)], true),
        ],
    }
}

pub fn get_gaols_algo1() -> Vec<SimulationConfig> {
    vec![
        // Frame cfg-loaded-1763382083818-6:
        SimulationConfig::new(
            vec![('R', 0, 1), ('F', -1, 1), ('O', 0, 0)],
            vec![(
                2,
                vec![('R', 2, 1), ('F', 1, 1), ('O', 0, 0)],
                vec![],
                vec![],
            )],
            Some((-2, 3, -1, 2)),
            (None, None),
        ),
        // Frame cfg-loaded-1763382083818-7:
        SimulationConfig::new(
            vec![('R', 0, -1), ('F', -1, -1), ('O', 0, 0)],
            vec![(
                2,
                vec![('R', 2, -1), ('F', 1, -1), ('O', 0, 0)],
                vec![],
                vec![],
            )],
            Some((-2, 3, -2, 1)),
            (None, None),
        ),
        // Frame cfg-loaded-1763382083818-8:
        SimulationConfig::new(
            vec![('L', 0, 1), ('F', -1, 1), ('O', 0, 0)],
            vec![(
                2,
                vec![('L', 2, 1), ('F', 1, 1), ('O', 0, 0)],
                vec![],
                vec![],
            )],
            Some((-2, 3, -1, 2)),
            (None, None),
        ),
        // Frame cfg-loaded-1763382083818-9:
        SimulationConfig::new(
            vec![('L', 0, -1), ('F', -1, -1), ('O', 0, 0)],
            vec![(
                2,
                vec![('L', 2, -1), ('F', 1, -1), ('O', 0, 0)],
                vec![],
                vec![],
            )],
            Some((-2, 3, -2, 1)),
            (None, None),
        ),
        // Frame cfg-loaded-1763382083818-10:
        SimulationConfig::new(
            vec![('R', -1, 0), ('F', -2, 0), ('O', 0, 0)],
            vec![(
                3,
                vec![('F', -2, 0), ('L', -3, 0), ('O', 0, 0)],
                vec![],
                vec![],
            )],
            Some((-4, 1, -2, 1)),
            (None, None),
        ),
        // Frame cfg-loaded-1763382083818-11:
        SimulationConfig::new(
            vec![('L', -1, 0), ('F', -2, 0), ('O', 0, 0)],
            vec![(
                4,
                vec![('L', 2, 1), ('F', 1, 1), ('O', 0, 0)],
                vec![],
                vec![],
            )],
            Some((-3, 3, -1, 2)),
            (None, None),
        ),
        // Frame cfg-new-1763382895396:
        SimulationConfig::new(
            vec![('R', -2, 0), ('F', -1, 0), ('O', 0, 0)],
            vec![(
                1,
                vec![('O', 0, 0), ('F', -2, 0), ('R', -3, 0)],
                vec![],
                vec![],
            )],
            Some((-4, 1, -1, 1)),
            (None, None),
        ),
        // Frame cfg-new-1763382934044:
        SimulationConfig::new(
            vec![('L', -2, 0), ('F', -1, 0), ('O', 0, 0)],
            vec![(
                1,
                vec![('L', -3, 0), ('F', -2, 0), ('O', 0, 0)],
                vec![],
                vec![],
            )],
            Some((-4, 1, -1, 1)),
            (None, None),
        ),
    ]
}

pub fn create_config_algo2() -> Config {
    Config {
        obstacle: 'O',
        number_of_robots: 2,
        number_of_colors: 2,
        visibility_range: 2,
        all_color_letters: vec!['F', 'L', 'R', 'Y', 'O'],
        existing_algorithm_path: "/src/data/algorithms_data2.json".to_string(),
        generation_mode: GenerationMode::ProgressiveValidationByLevels(4),
        opacity: true,
        is_obstacle_opaque: true,
        web_algo_colors: "  L: 16711680\n  F: 255\n  O: 16753920".to_string(),

        web_algo_initial_configuration: ".\nWWWWWWWWWWWWWWW\nW.............W\nW.............W\nW.............W\nW.............W\nW...F.L.......W\nW.............W\nW......O......W\nW.............W\nW.............W\nW.............W\nW.............W\nW.............W\nW.............W\nWWWWWWWWWWWWWWW".to_string(),
        web_algo_walls: vec![[0, 0], [17, 17]],
        leader_colors: vec!['L'],
        moving_on_space_pattern: vec![vec![('L', 0, 0), ('F', -1, 0)], vec![('L', 0, 0), ('F', -2, 0)]],
        goals: get_gaols_algo2(),
        initial_configurations: vec![
            (vec![('O', 0, 0), ('L', 1, 2), ('F', 0, 2)], false),
            (vec![('O', 0, 0), ('L', 0, 2), ('F', -1, 2)], true),
            (vec![('O', 0, 0), ('L', 2, 1), ('F', 1, 1)], false),
            (vec![('O', 0, 0), ('L', 1, 1), ('F', 0, 1)], false),
            (vec![('O', 0, 0), ('L', 0, 1), ('F', -1, 1)], false),
            (vec![('O', 0, 0), ('L', -1, 1), ('F', -2, 1)], true),
            (vec![('O', 0, 0), ('L', 3, 0), ('F', 2, 0)], false),
            (vec![('O', 0, 0), ('L', 2, 0), ('F', 1, 0)], false),
            (vec![('O', 0, 0), ('L', -1, 0), ('F', -2, 0)], false),
            (vec![('O', 0, 0), ('L', -2, 0), ('F', -3, 0)], true),
            (vec![('O', 0, 0), ('L', 2, -1), ('F', 1, -1)], false),
            (vec![('O', 0, 0), ('L', 1, -1), ('F', 0, -1)], false),
            (vec![('O', 0, 0), ('L', 0, -1), ('F', -1, -1)], false),
            (vec![('O', 0, 0), ('L', -1, -1), ('F', -2, -1)], true),
            (vec![('O', 0, 0), ('L', 1, -2), ('F', 0, -2)], false),
            (vec![('O', 0, 0), ('L', 0, -2), ('F', -1, -2)], true),
            (vec![('O', 0, 0), ('L', 2, 2), ('F', 0, 2)], false),
            (vec![('O', 0, 0), ('L', 0, 2), ('F', -2, 2)], true),
            (vec![('O', 0, 0), ('L', 3, 1), ('F', 1, 1)], false),
            (vec![('O', 0, 0), ('L', 2, 1), ('F', 0, 1)], false),
            (vec![('O', 0, 0), ('L', 1, 1), ('F', -1, 1)], false),
            (vec![('O', 0, 0), ('L', 0, 1), ('F', -2, 1)], false),
            (vec![('O', 0, 0), ('L', -1, 1), ('F', -3, 1)], true),
            (vec![('O', 0, 0), ('L', 4, 0), ('F', 2, 0)], false),
            (vec![('O', 0, 0), ('L', 3, 0), ('F', 1, 0)], false),
            (vec![('O', 0, 0), ('L', -1, 0), ('F', -3, 0)], false),
            (vec![('O', 0, 0), ('L', -2, 0), ('F', -4, 0)], true),
            (vec![('O', 0, 0), ('L', 3, -1), ('F', 1, -1)], false),
            (vec![('O', 0, 0), ('L', 2, -1), ('F', 0, -1)], false),
            (vec![('O', 0, 0), ('L', 1, -1), ('F', -1, -1)], false),
            (vec![('O', 0, 0), ('L', 0, -1), ('F', -2, -1)], false),
            (vec![('O', 0, 0), ('L', -1, -1), ('F', -3, -1)], true),
            (vec![('O', 0, 0), ('L', 2, -2), ('F', 0, -2)], false),
            (vec![('O', 0, 0), ('L', 0, -2), ('F', -2, -2)], true),
        ],
    }
}

pub fn get_gaols_algo2() -> Vec<SimulationConfig> {
    vec![
        // Frame cfg-loaded-1760517281733-0:
        SimulationConfig::new(
            vec![('O', 0, 0), ('L', 0, 2), ('F', -2, 2)],
            vec![(
                3,
                vec![('L', 3, 2), ('F', 1, 2), ('O', 0, 0)],
                vec![],
                vec![],
            )],
            Some((-3, 4, -1, 3)),
            (None, None),
        ),
        // Frame cfg-loaded-1760517281733-1:
        SimulationConfig::new(
            vec![('O', 0, 0), ('L', 0, -2), ('F', -2, -2)],
            vec![(
                3,
                vec![('O', 0, 0), ('L', 3, -2), ('F', 1, -2)],
                vec![],
                vec![],
            )],
            Some((-3, 4, -3, 1)),
            (None, None),
        ),
        // Frame cfg-loaded-1760517281733-2:
        SimulationConfig::new(
            vec![('L', -1, 1), ('O', 0, 0), ('F', -3, 1)],
            vec![(
                5,
                vec![('F', 2, 1), ('L', 4, 1), ('O', 0, 0)],
                vec![],
                vec![],
            )],
            Some((-4, 5, -1, 2)),
            (None, None),
        ),
        // Frame cfg-loaded-1760517281733-3:
        SimulationConfig::new(
            vec![('O', 0, 0), ('L', -1, -1), ('F', -3, -1)],
            vec![(
                5,
                vec![('L', 4, -1), ('F', 2, -1), ('O', 0, 0)],
                vec![],
                vec![],
            )],
            Some((-4, 5, -2, 1)),
            (None, None),
        ),
        // Frame cfg-loaded-1760517281733-4:
        SimulationConfig::new(
            vec![('O', 0, 0), ('L', 0, 2), ('F', -1, 2)],
            vec![(
                2,
                vec![('O', 0, 0), ('F', 1, 2), ('L', 2, 2)],
                vec![],
                vec![],
            )],
            Some((-2, 3, -1, 3)),
            (None, None),
        ),
        // Frame cfg-loaded-1760517281733-5:
        SimulationConfig::new(
            vec![('O', 0, 0), ('L', 0, -2), ('F', -1, -2)],
            vec![(
                2,
                vec![('O', 0, 0), ('F', 1, -2), ('L', 2, -2)],
                vec![],
                vec![],
            )],
            Some((-2, 3, -3, 1)),
            (None, None),
        ),
        // Frame cfg-loaded-1760517281733-6:
        SimulationConfig::new(
            vec![('O', 0, 0), ('L', -1, 1), ('F', -2, 1)],
            vec![(
                4,
                vec![('O', 0, 0), ('F', 2, 1), ('L', 3, 1)],
                vec![],
                vec![],
            )],
            Some((-3, 4, -1, 2)),
            (None, None),
        ),
        // Frame cfg-loaded-1760517281733-7:
        SimulationConfig::new(
            vec![('O', 0, 0), ('F', -2, -1), ('L', -1, -1)],
            vec![(
                4,
                vec![('O', 0, 0), ('L', 3, -1), ('F', 2, -1)],
                vec![],
                vec![],
            )],
            Some((-3, 4, -2, 1)),
            (None, None),
        ),
        // Frame cfg-loaded-1760517281733-8:
        SimulationConfig::new(
            vec![('O', 0, 0), ('F', -4, 0), ('L', -2, 0)],
            vec![(
                2,
                vec![('O', 0, 0), ('L', -1, 1), ('F', -3, 1)],
                vec![],
                vec![(-1, 0)],
            )],
            Some((-5, 1, -1, 2)),
            (None, None),
        ),
        // Frame cfg-loaded-1760517281733-9:
        SimulationConfig::new(
            vec![('O', 0, 0), ('L', -2, 0), ('F', -3, 0)],
            vec![(
                4,
                vec![('O', 0, 0), ('F', -3, 0), ('L', -5, 0)],
                vec![],
                vec![],
            )],
            Some((-6, 1, -1, 1)),
            (None, None),
        ),
        // Frame cfg-loaded-1760517281733-10:
        SimulationConfig::new(
            vec![('O', 0, 0), ('F', -2, 0), ('L', -1, 0)],
            vec![(
                2,
                vec![('O', 0, 0), ('F', -1, 1), ('L', -2, 1)],
                vec![],
                vec![],
            )],
            Some((-3, 1, -1, 2)),
            (None, None),
        ),
    ]
}
