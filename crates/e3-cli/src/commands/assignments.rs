use crate::config::build_client_with_relogin;
use crate::output;
use colored::Colorize;
use comfy_table::{presets, ContentArrangement, Table};
use e3_core::error::Result;

pub async fn run(json: bool, base_url: Option<&str>, days: i64) -> Result<()> {
    let (client, _config) = build_client_with_relogin(base_url).await?;

    let sp = if !json {
        Some(output::spinner("取得作業..."))
    } else {
        None
    };

    let assignments =
        e3_core::assignments::get_pending_assignments_via_calendar(&client, days).await?;

    if let Some(sp) = sp {
        sp.finish_and_clear();
    }

    if json {
        output::print_json_success(&assignments);
        return Ok(());
    }

    if assignments.is_empty() {
        println!("{}", "✓ 沒有待繳作業".green());
        return Ok(());
    }

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL_CONDENSED);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["ID", "課程", "作業名稱", "截止日期", "狀態"]);

    for a in &assignments {
        let date_str = a
            .duedate
            .map(output::format_date)
            .unwrap_or_else(|| "無期限".into());

        let status = if a.is_overdue {
            "逾期".red().to_string()
        } else {
            match a.submission_status.as_str() {
                "new" => "未繳".yellow().to_string(),
                "draft" => "草稿".blue().to_string(),
                _ => a.submission_status.clone(),
            }
        };

        table.add_row(vec![
            a.id.to_string(),
            a.course_shortname.clone(),
            a.name.clone(),
            date_str,
            status,
        ]);
    }

    println!("{table}");
    println!(
        "{}",
        format!("共 {} 項待繳作業", assignments.len()).dimmed()
    );

    Ok(())
}

pub async fn submission(json: bool, base_url: Option<&str>, cmid: i64) -> Result<()> {
    let (client, _config) = build_client_with_relogin(base_url).await?;

    let sp = if !json {
        Some(output::spinner("取得提交狀態..."))
    } else {
        None
    };

    // Resolve cmid to assign id
    let (assign_id, _course_id) = e3_core::assignments::resolve_assign_id(&client, cmid).await?;
    let status = e3_core::assignments::get_submission_status(&client, assign_id).await?;

    if let Some(sp) = sp {
        sp.finish_and_clear();
    }

    if json {
        output::print_json_success(&status);
        return Ok(());
    }

    // Display submission info
    if let Some(la) = &status.lastattempt {
        let sub = la.submission.as_ref().or(la.teamsubmission.as_ref());

        if let Some(sub) = sub {
            let status_str = sub.status.as_deref().unwrap_or("unknown");
            let status_colored = match status_str {
                "submitted" => status_str.green().to_string(),
                "draft" => status_str.blue().to_string(),
                "new" => status_str.yellow().to_string(),
                _ => status_str.to_string(),
            };

            println!("{}: {}", "提交狀態".bold(), status_colored);
            println!(
                "{}: {}",
                "提交時間".bold(),
                sub.timemodified
                    .map(output::format_date)
                    .unwrap_or_else(|| "—".into())
            );

            // Show uploaded files
            for plugin in &sub.plugins {
                for area in &plugin.fileareas {
                    if !area.files.is_empty() {
                        println!("\n{}", "已上傳檔案:".bold());
                        for f in &area.files {
                            println!(
                                "  {} {} ({})",
                                "•".dimmed(),
                                f.filename.as_deref().unwrap_or("?"),
                                format_size(f.filesize.unwrap_or(0)),
                            );
                        }
                    }
                }
            }
        } else {
            println!("{}: {}", "提交狀態".bold(), "尚未提交".yellow());
        }

        if let Some(grading) = &la.gradingstatus {
            println!("{}: {}", "批改狀態".bold(), grading);
        }
    }

    // Display feedback
    if let Some(fb) = &status.feedback {
        if let Some(grade_display) = &fb.gradefordisplay {
            println!("\n{}", "成績回饋:".bold().cyan());
            println!("{}: {}", "成績".bold(), output::strip_html(grade_display));
        }
        if let Some(date) = fb.gradeddate {
            println!("{}: {}", "批改日期".bold(), output::format_date(date));
        }
    }

    Ok(())
}

fn format_size(bytes: i64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
