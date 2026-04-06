use colored::Colorize;
use e3_core::error::{E3Error, ErrorInfo};

/// Print JSON success response
pub fn print_json_success(data: &impl serde::Serialize) {
    let output = serde_json::json!({
        "success": true,
        "data": data,
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

/// Print JSON error response
pub fn print_json_error(err: &E3Error) {
    let info = ErrorInfo::from(err);
    let output = serde_json::json!({
        "success": false,
        "error": info,
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

/// Print error to stderr
pub fn print_error(err: &E3Error) {
    eprintln!("{} {}", "error:".red().bold(), err);
}

/// Format a Unix timestamp to human-readable date string (Asia/Taipei)
pub fn format_date(timestamp: i64) -> String {
    if timestamp == 0 {
        return "無期限".into();
    }
    let dt = chrono::DateTime::from_timestamp(timestamp, 0);
    match dt {
        Some(dt) => {
            let taipei = chrono_tz::Asia::Taipei;
            let local = dt.with_timezone(&taipei);
            local.format("%Y-%m-%d %H:%M").to_string()
        }
        None => "Invalid".into(),
    }
}

/// Get urgency color for a due date
pub fn urgency_color(timestamp: i64) -> colored::Color {
    if timestamp == 0 {
        return colored::Color::Green;
    }
    let now = chrono::Utc::now().timestamp();
    let diff = timestamp - now;

    if diff < 86400 {
        colored::Color::Red // overdue or < 24h
    } else if diff < 259200 {
        colored::Color::Yellow // < 72h
    } else {
        colored::Color::Green
    }
}

/// Create a spinner with a message
pub fn spinner(msg: &str) -> indicatif::ProgressBar {
    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_style(
        indicatif::ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}

/// Create a progress bar for downloads
#[allow(dead_code)]
pub fn progress_bar(total: u64) -> indicatif::ProgressBar {
    let pb = indicatif::ProgressBar::new(total);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb
}

/// Strip HTML for display
pub fn strip_html(html: &str) -> String {
    e3_core::ics::strip_html(html)
}
