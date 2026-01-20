use std::{collections::HashMap, fs, io::Write};

pub struct WebAlgoGenerator {
    filename: String,
    output_dir: String,
    template: String,
    data: HashMap<String, String>,
}

impl WebAlgoGenerator {
    pub fn new(filename: &str, output_dir: &str, template: &str) -> Self {
        Self {
            filename: filename.to_string(),
            output_dir: output_dir.to_string(),
            template: template.to_string(),
            data: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    pub fn add_rule(&mut self, rule: &str) {
        self.data
            .entry("rules".to_string())
            .or_insert_with(String::new)
            .push_str(&format!("\n{}", rule));
    }

    pub fn save_web_algo(&self) {
        let output = self
            .data
            .iter()
            .fold(self.template.clone(), |acc, (key, value)| {
                acc.replace(&format!("{{{}}}", key), value)
            });
        fs::create_dir_all(&self.output_dir)
            .unwrap_or_else(|e| eprintln!("Failed to create dir: {}", e));
        fs::write(
            format!("{}/{}.web-algo", self.output_dir, self.filename),
            output,
        )
        .unwrap_or_else(|e| eprintln!("Failed to write file: {}", e));
    }
}
