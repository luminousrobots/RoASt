use std::fs;

use chrono::Local;

pub struct FolderGenerator;

impl FolderGenerator {
    pub fn create_folder(base_dir: &str, prefix: &str) -> String {
        let output_dir = format!("{}/{}", base_dir, prefix);
        fs::create_dir_all(&output_dir).expect("Failed to create output directory");
        output_dir
    }
}
