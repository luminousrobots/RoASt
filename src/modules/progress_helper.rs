use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct ProgressHelper {
    pub bar: ProgressBar,
    pub counter: Arc<AtomicUsize>,
}

impl ProgressHelper {
    /// Create a standard progress bar with consistent styling
    pub fn new(total: u64, operation: &str) -> Self {
        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(&format!(
                    "🔄 {}: [{{bar:40.cyan/blue}}] {{pos}}/{{len}} ({{percent}}%) | ETA: {{eta}}",
                    operation
                ))
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏ "),
        );

        // Don't print starting message - it's redundant with the progress bar

        Self {
            bar: pb,
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Create a progress bar with custom message updates
    pub fn new_with_messages(total: u64, operation: &str) -> Self {
        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("🔄 Processing: [{{bar:40.cyan/blue}}] {{pos}}/{{len}} | {{per_sec}} | ETA: {{eta}} | {{msg}}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏ ")
        );

        pb.set_message(format!("Processing..."));

        Self {
            bar: pb,
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Increment progress (thread-safe)
    pub fn inc(&self) {
        self.bar.inc(1);
        self.counter.fetch_add(1, Ordering::Relaxed);
    }

    /// Update progress with a message
    pub fn update_message(&self, msg: &str) {
        self.bar.set_message(msg.to_string());
    }

    /// Finish with success message
    pub fn finish_success(&self, operation: &str, total: usize) {
        self.bar.finish_with_message("✅ Completed successfully!");
        println!("📊 {} completed: {} items processed", operation, total);
    }

    /// Finish with custom message
    pub fn finish_with(&self, message: &str) {
        self.bar.finish_with_message(message.to_string());
    }

    /// Get current progress count
    pub fn current_count(&self) -> usize {
        self.counter.load(Ordering::Relaxed)
    }
}

/// Helper trait for parallel processing with progress
pub trait ParallelProgressExt<T> {
    fn par_process_with_progress<F, R>(&self, operation: &str, processor: F) -> Vec<R>
    where
        F: Fn(&T) -> R + Sync + Send,
        R: Send;
}

impl<T: Sync> ParallelProgressExt<T> for [T] {
    fn par_process_with_progress<F, R>(&self, operation: &str, processor: F) -> Vec<R>
    where
        F: Fn(&T) -> R + Sync + Send,
        R: Send,
    {
        let progress = ProgressHelper::new(self.len() as u64, operation);

        let result: Vec<R> = self
            .par_iter()
            .map(|item| {
                let result = processor(item);
                progress.inc();
                result
            })
            .collect();

        progress.finish_success(operation, self.len());
        result
    }
}
