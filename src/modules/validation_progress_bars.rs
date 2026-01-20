use std::{
    fs::File,
    io::Write, // <-- Add this line
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

use chrono::Local;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::modules::{algorithm_stats::AlgorithmStats, config_stats::ConfigStats};

pub struct ValidationProgressBars {
    pub algo: ProgressBar,
    pub config: ProgressBar,
    pub status: ProgressBar,
    pub start_time: Instant,
}

pub fn create_progress_bars(
    num_algos: usize,
    num_configs_per_algo: usize,
) -> ValidationProgressBars {
    let m = MultiProgress::new();

    let algo = m.add(ProgressBar::new(num_algos as u64));
    algo.set_style(
        ProgressStyle::default_bar()
            .template("{bar:40.cyan/blue} {pos}/{len} algorithms")
            .unwrap(),
    );
    algo.set_position(0); // Force initial render of algorithm bar

    let config = m.add(ProgressBar::new((num_algos * num_configs_per_algo) as u64));
    config.set_style(
        ProgressStyle::default_bar()
            .template("{bar:40.yellow/red} Configs: {pos}/{len} | {per_sec} | ETA: {eta}")
            .unwrap(),
    );
    config.set_position(0); // Force initial render of config bar

    let status = m.add(ProgressBar::new(0));
    status.set_style(
        ProgressStyle::default_bar()
            .template("⟳ Status: {msg}")
            .unwrap(),
    );
    //status.set_draw_target(indicatif::ProgressDrawTarget::hidden());  // Hide status bar

    ValidationProgressBars {
        algo,
        config,
        status,
        start_time: Instant::now(),
    }
}

fn format_duration(elapsed_secs: u64) -> (String, String) {
    let hours = elapsed_secs / 3600;
    let minutes = (elapsed_secs % 3600) / 60;
    let seconds = elapsed_secs % 60;

    let short_format = if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    };

    let long_format = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);

    (short_format, long_format)
}

fn get_current_time() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn start_status_updater(
    pb_algo: ProgressBar,
    pb_status: ProgressBar,
    algo_stats: Arc<AlgorithmStats>,
    base_path: String,
    start_time: Instant,
) {
    std::thread::spawn(move || {
        let log_path = format!("{}/_progress.log", base_path);

        while !pb_algo.is_finished() {
            // Calculate algorithm-level statistics
            let algo_snapshot = algo_stats.snapshot();
            let vld_alg = algo_snapshot.validated_ld.len();
            let vnld_alg = algo_snapshot.validated_not_ld.len();
            let blocked_alg = algo_snapshot.blocked.len();
            let cyclic_alg = algo_snapshot.cyclic.len();
            let timeout_alg = algo_snapshot.timeout.len();
            let completed_algos = vld_alg + vnld_alg + blocked_alg + cyclic_alg + timeout_alg;
            let current = pb_algo.position() as usize;
            let total = pb_algo.length().unwrap_or(0) as usize;

            let elapsed = start_time.elapsed().as_secs();
            let (duration_short, duration_long) = format_duration(elapsed);
            let current_time = get_current_time();

            let status_msg = format!(
                "algos {} (VLD {} | NOT-LD {} | ⊗ {} | ⟲ {} | ⏱ {})",
                completed_algos, vld_alg, vnld_alg, blocked_alg, cyclic_alg, timeout_alg
            );

            //pb_status.set_message(status_msg.clone());
            pb_status.set_message(status_msg);
            pb_algo.tick();

            // Write to file
            if let Ok(mut file) = File::create(&log_path) {
                writeln!(
                    file,
                    "┌──────────────────────────────────────────────────────────────┐"
                )
                .ok();
                writeln!(
                    file,
                    "│                   📊 VALIDATION IN PROGRESS                  │"
                )
                .ok();
                writeln!(
                    file,
                    "├──────────────────────────────────────────────────────────────┤"
                )
                .ok();
                writeln!(file, "│ Current Time : {:<45} │", current_time).ok();
                writeln!(
                    file,
                    "│ Duration     : {} ({} sec){:<28} │",
                    duration_long, elapsed, ""
                )
                .ok();
                writeln!(
                    file,
                    "├──────────────────────────────────────────────────────────────┤"
                )
                .ok();
                writeln!(
                    file,
                    "│ Progress     : {}/{} algorithms ({:.1}%){:<24} │",
                    current,
                    total,
                    (current as f64 / total as f64) * 100.0,
                    ""
                )
                .ok();
                writeln!(
                    file,
                    "├──────────────────────────────────────────────────────────────┤"
                )
                .ok();
                writeln!(file, "│ ✓ Validated (LD)     : {:<39} │", vld_alg).ok();
                writeln!(file, "│ ✓ Validated (NOT-LD) : {:<39} │", vnld_alg).ok();
                writeln!(file, "│ ⊗ Blocked            : {:<39} │", blocked_alg).ok();
                writeln!(file, "│ ⟲ Cyclic             : {:<39} │", cyclic_alg).ok();
                writeln!(file, "│ ⏱ Timeout            : {:<39} │", timeout_alg).ok();
                writeln!(
                    file,
                    "└──────────────────────────────────────────────────────────────┘"
                )
                .ok();

                file.flush().ok();
            }

            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    });
}

pub fn finish_progress_bars(
    progress_bars: &ValidationProgressBars,
    validated_ld_count: usize,
    validated_not_ld_count: usize,
    blocked_count: usize,
    cyclic_count: usize,
    timeout_count: usize,
    total_algos: usize,
    base_path: &str,
) {
    progress_bars.algo.finish();
    progress_bars.config.finish();

    let final_msg = format!(
        "✓ {} VLD | ✓ {} VNLD | ⊗ {} blocked | ⟲ {} cyclic | ⏱ {} timeout",
        validated_ld_count, validated_not_ld_count, blocked_count, cyclic_count, timeout_count
    );

    progress_bars.status.finish_with_message(final_msg.clone());

    // Write final status
    let log_path = format!("{}/_progress.log", base_path);
    let elapsed_secs = progress_bars.start_time.elapsed().as_secs();
    let elapsed_millis = progress_bars.start_time.elapsed().as_millis();
    let elapsed_float = elapsed_millis as f64 / 1000.0;
    let (_, duration_long) = format_duration(elapsed_secs);
    let end_time = get_current_time();

    if let Ok(mut file) = File::create(&log_path) {
        writeln!(
            file,
            "┌──────────────────────────────────────────────────────────────┐"
        )
        .ok();
        writeln!(
            file,
            "│                    🏁 VALIDATION COMPLETED                   │"
        )
        .ok();
        writeln!(
            file,
            "├──────────────────────────────────────────────────────────────┤"
        )
        .ok();
        writeln!(file, "│ End Time     : {:<45} │", end_time).ok();
        writeln!(
            file,
            "│ Duration     : {} ({:.3} sec){:<23} │",
            duration_long, elapsed_float, ""
        )
        .ok();
        writeln!(
            file,
            "├──────────────────────────────────────────────────────────────┤"
        )
        .ok();
        writeln!(
            file,
            "│ Total        : {}/{} algorithms (100%){:<24} │",
            total_algos, total_algos, ""
        )
        .ok();
        writeln!(
            file,
            "├──────────────────────────────────────────────────────────────┤"
        )
        .ok();
        writeln!(
            file,
            "│ ✓ Validated (LD)     : {:<39} │",
            validated_ld_count
        )
        .ok();
        writeln!(
            file,
            "│ ✓ Validated (NOT-LD) : {:<39} │",
            validated_not_ld_count
        )
        .ok();
        writeln!(file, "│ ⊗ Blocked            : {:<39} │", blocked_count).ok();
        writeln!(file, "│ ⟲ Cyclic             : {:<39} │", cyclic_count).ok();
        writeln!(file, "│ ⏱ Timeout            : {:<39} │", timeout_count).ok();
        writeln!(
            file,
            "└──────────────────────────────────────────────────────────────┘"
        )
        .ok();

        file.flush().ok();
    }
}
