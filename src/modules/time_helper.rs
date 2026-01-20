use std::time::Instant;

/// Format elapsed time in a human-readable way
/// Examples:
/// - 45.5 sec -> "45.5 sec"
/// - 90 sec -> "1 min 30 sec"
/// - 3661.5 sec -> "1h 1min 1.5 sec"
/// - 10923.2222 sec -> "3h 2min 3.2 sec"
pub fn format_elapsed_time(start_time: Instant) -> String {
    let seconds = start_time.elapsed().as_secs_f32();

    let total_seconds = seconds as u64;
    let remaining_seconds = seconds - total_seconds as f32;

    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let secs = total_seconds % 60;
    let decimal_secs = secs as f32 + remaining_seconds;

    if hours > 0 {
        format!("({}h {}min {:.4} sec)", hours, minutes, decimal_secs)
    } else if minutes > 0 {
        format!("({} min {:.4} sec)", minutes, decimal_secs)
    } else {
        format!("({:.5} sec)", seconds)
    }
}
