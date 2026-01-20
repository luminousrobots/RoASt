use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

pub const CACHE_DIR: &str = "src/cache";

pub fn ensure_cache_dir() {
    if !Path::new(CACHE_DIR).exists() {
        fs::create_dir_all(CACHE_DIR).unwrap();
    }
}

pub fn save_json<T: ?Sized + Serialize>(
    file: &str,
    data: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    ensure_cache_dir();
    let path = format!("{}/{}", CACHE_DIR, file);
    let json = serde_json::to_string_pretty(data)?;
    let mut f = fs::File::create(path)?;
    f.write_all(json.as_bytes())?;
    Ok(())
}

pub fn load_json<T: DeserializeOwned>(file: &str) -> Option<T> {
    let path = format!("{}/{}", CACHE_DIR, file);
    if Path::new(&path).exists() {
        let data = fs::read_to_string(path).unwrap();
        Some(serde_json::from_str(&data).unwrap())
    } else {
        None
    }
}

pub fn clean_cache() -> bool {
    if Path::new(CACHE_DIR).exists() {
        if let Err(e) = fs::remove_dir_all(CACHE_DIR) {
            println!("❌ Failed to clear cache: {}", e);
            return false;
        }
    }
    // Recreate the cache directory
    if let Err(e) = fs::create_dir_all(CACHE_DIR) {
        println!("❌ Failed to recreate cache dir: {}", e);
        return false;
    }
    // Create .gitkeep file
    let gitkeep_path = format!("{}/.gitkeep", CACHE_DIR);
    if let Err(e) = File::create(&gitkeep_path) {
        println!("❌ Failed to create .gitkeep: {}", e);
        return false;
    }
    println!("🗑️ Cache cleared and .gitkeep restored.");
    true
}
