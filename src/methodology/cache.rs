use super::globals::*;
use crate::modules::cache_helpers::{clean_cache, load_json, save_json};

/// Save all - returns true if all saves successful
pub fn save_all() -> bool {
    macro_rules! try_save {
        ($file:expr, $data:expr) => {
            match save_json($file, $data) {
                Ok(_) => true,
                Err(_) => {
                    println!("❌ Failed to save {}", $file);
                    false
                }
            }
        };
    }

    let results = [
        try_save!("views.json", &*get_views()),
        try_save!("rules.json", &*get_rules()),
        try_save!("parallel_rules.json", &*get_parallel_rules()),
        try_save!("robots.json", &*get_number_of_robots()),
        try_save!("colors.json", &*get_number_of_colors()),
        try_save!("visibility.json", &*get_visibility()),
        try_save!("letters.json", &get_all_color_letters()),
        try_save!("original_rules_count.json", &get_original_rules_count()),
        try_save!("original_views_count.json", &get_original_views_count()),
    
    ];

    let all_success = results.iter().all(|&r| r);
    if all_success {
        println!("✅ All {} files saved successfully", results.len());
    }
    all_success
}

/// Load all (prints when missing) - returns true if all files loaded
pub fn load_all() -> bool {
    // First check if cached feature matches current feature

    macro_rules! try_load {
        ($file:expr, $setter:expr) => {
            match load_json($file) {
                Some(data) => {
                    $setter(data);
                    println!("✅ Loaded {}", $file);
                    true
                }
                None => {
                    println!("ℹ️ No cache for {}", $file);
                    false
                }
            }
        };
    }

    let results = [
        try_load!("views.json", set_views),
        try_load!("rules.json", set_rules),
        try_load!("parallel_rules.json", set_parallel_rules),
        try_load!("robots.json", set_number_of_robots),
        try_load!("colors.json", set_number_of_colors),
        try_load!("visibility.json", set_visibility),
        try_load!("letters.json", set_all_color_letters),
        try_load!("original_rules_count.json", set_original_rules_count),
        try_load!("original_views_count.json", set_original_views_count),
    ];

    let loaded = results.iter().filter(|&&r| r).count();
    println!(
        "📊 Cache summary: {}/{} files loaded",
        loaded,
        results.len()
    );
    loaded == results.len()
}

/// Clean all cache files - returns true if successful
pub fn clean_all() -> bool {
    clean_cache()
}
