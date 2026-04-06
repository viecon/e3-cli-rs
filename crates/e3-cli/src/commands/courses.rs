use crate::config::build_client_with_relogin;
use crate::output;
use colored::Colorize;
use comfy_table::{presets, ContentArrangement, Table};
use e3_core::error::Result;

pub async fn run(json: bool, base_url: Option<&str>, all: bool) -> Result<()> {
    let (client, _config) = build_client_with_relogin(base_url).await?;

    let sp = if !json {
        Some(output::spinner("取得課程..."))
    } else {
        None
    };

    let classification = if all { "all" } else { "inprogress" };
    let courses = e3_core::courses::get_enrolled_courses(&client, classification).await?;

    if let Some(sp) = sp {
        sp.finish_and_clear();
    }

    if json {
        output::print_json_success(&courses);
        return Ok(());
    }

    if courses.is_empty() {
        println!("{}", "沒有課程".dimmed());
        return Ok(());
    }

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL_CONDENSED);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["ID", "簡稱", "課程名稱", "進度"]);

    for c in &courses {
        let progress = c
            .progress
            .map(|p| format!("{:.0}%", p))
            .unwrap_or_else(|| "—".into());

        table.add_row(vec![
            c.id.to_string(),
            c.shortname.clone().unwrap_or_default(),
            c.fullname.clone().unwrap_or_default(),
            progress,
        ]);
    }

    println!("{table}");
    println!("{}", format!("共 {} 門課程", courses.len()).dimmed());

    Ok(())
}
