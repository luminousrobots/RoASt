use chrono::Local;
use once_cell::sync::Lazy;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime};

pub use crate::modules::time_helper::format_elapsed_time;

pub struct ExecutionLogger {
    start_time: Instant,
    system_start_time: SystemTime,
    pub file_path: String,
}

impl ExecutionLogger {
    /// Create a logger with a custom directory and filename prefix
    pub fn start(log_dir: &str, filename_prefix: &str) -> Self {
        // Ensure directory exists
        create_dir_all(log_dir).expect("Failed to create results directory");

        // Timestamped file name with prefix
        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        let file_path = format!("{}/{}_{}.log", log_dir, filename_prefix, timestamp);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .unwrap();

        let now = Local::now();
        writeln!(
            file,
            "\n┌──────────────────────────────────────────────────────────────┐"
        )
        .unwrap();
        writeln!(
            file,
            "│                    🚀 EXECUTION STARTED                      │"
        )
        .unwrap();
        writeln!(
            file,
            "├──────────────────────────────────────────────────────────────┤"
        )
        .unwrap();
        writeln!(
            file,
            "│ Time      : {}                              │",
            now.format("%Y-%m-%d %H:%M:%S")
        )
        .unwrap();
        writeln!(
            file,
            "└──────────────────────────────────────────────────────────────┘\n"
        )
        .unwrap();

        ExecutionLogger {
            start_time: Instant::now(),
            system_start_time: SystemTime::now(),
            file_path,
        }
    }

    pub fn log_note<S: AsRef<str>>(&self, note: S) {
        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.file_path)
            .unwrap();

        writeln!(
            file,
            "⏱ [{}]  •  {}",
            format_elapsed_time(self.start_time),
            note.as_ref()
        )
        .unwrap();
    }

    pub fn end(&self) {
        let end_time = Instant::now();

        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.file_path)
            .unwrap();

        let now = Local::now();
        writeln!(
            file,
            "\n┌──────────────────────────────────────────────────────────────┐"
        )
        .unwrap();
        writeln!(
            file,
            "│                    🏁 EXECUTION COMPLETED                    │"
        )
        .unwrap();
        writeln!(
            file,
            "├──────────────────────────────────────────────────────────────┤"
        )
        .unwrap();
        writeln!(
            file,
            "│ End Time : {}                               │",
            now.format("%Y-%m-%d %H:%M:%S")
        )
        .unwrap();
        writeln!(
            file,
            "│ Duration : {:<50}│",
            format_elapsed_time(self.start_time),
        )
        .unwrap();
        writeln!(
            file,
            "└──────────────────────────────────────────────────────────────┘\n"
        )
        .unwrap();
    }
}

// === Global Singleton ===
pub static LOGGER: Lazy<Mutex<Option<ExecutionLogger>>> = Lazy::new(|| Mutex::new(None));

/// Initializes the logger with a directory and filename prefix.
/// Returns the full path to the log file.
pub fn init_logger(log_dir: &str, filename_prefix: &str) -> String {
    let logger = ExecutionLogger::start(log_dir, filename_prefix);
    let path = logger.file_path.clone();

    let mut global_logger = LOGGER.lock().unwrap();
    *global_logger = Some(logger);

    path
}

pub fn log_note(note: &str) {
    if let Some(ref logger) = *LOGGER.lock().unwrap() {
        logger.log_note(note);
    }
}

pub fn end_logger() {
    if let Some(ref logger) = *LOGGER.lock().unwrap() {
        logger.end();
    }
}

pub fn get_logger_path() -> Option<String> {
    LOGGER
        .lock()
        .unwrap()
        .as_ref()
        .map(|logger| logger.file_path.clone())
}
