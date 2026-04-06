use crate::config::E3Config;
use colored::Colorize;
use e3_core::error::{E3Error, Result};

pub fn run(base_url: Option<&str>, target: Option<String>) -> Result<()> {
    let config = E3Config::load().unwrap_or_default();
    let base = base_url
        .or(config.base_url.as_deref())
        .unwrap_or("https://e3p.nycu.edu.tw");

    let url = match target.as_deref() {
        None | Some("dashboard") | Some("home") => format!("{base}/my/"),
        Some("calendar") => format!("{base}/calendar/view.php?view=month"),
        Some("grades") => format!("{base}/grade/report/overview/index.php"),
        Some(t) => {
            // Try as course ID
            if let Ok(id) = t.parse::<i64>() {
                format!("{base}/course/view.php?id={id}")
            } else {
                // Try as a generic path
                format!("{base}/{t}")
            }
        }
    };

    open::that(&url).map_err(|e| E3Error::Other(format!("Cannot open browser: {e}")))?;
    println!("{} {}", "✓ 已開啟:".green().bold(), url);

    Ok(())
}
