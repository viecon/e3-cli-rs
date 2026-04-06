use crate::config::build_client_with_relogin;
use crate::output;
use colored::Colorize;
use comfy_table::{presets, ContentArrangement, Table};
use e3_core::error::{E3Error, Result};
use e3_core::files;
use std::path::PathBuf;

pub async fn run(
    json: bool,
    base_url: Option<&str>,
    days: i64,
    course: Option<i64>,
    all: bool,
) -> Result<()> {
    let (client, _config) = build_client_with_relogin(base_url).await?;

    let sp = if !json {
        Some(output::spinner("取得作業..."))
    } else {
        None
    };

    let assignments = if let Some(course_id) = course {
        // Course-specific: use REST API for richer data
        if all {
            e3_core::assignments::get_all_course_assignments(&client, &[course_id]).await?
        } else {
            e3_core::assignments::get_pending_assignments(&client, &[course_id]).await?
        }
    } else if all {
        // All courses + all assignments
        let courses = e3_core::courses::get_enrolled_courses(&client, "inprogress").await?;
        let course_ids: Vec<i64> = courses.iter().map(|c| c.id).collect();
        e3_core::assignments::get_all_course_assignments(&client, &course_ids).await?
    } else {
        // Default: pending only via calendar
        e3_core::assignments::get_pending_assignments_via_calendar(&client, days).await?
    };

    if let Some(sp) = sp {
        sp.finish_and_clear();
    }

    if json {
        output::print_json_success(&assignments);
        return Ok(());
    }

    if assignments.is_empty() {
        println!("{}", "✓ 沒有作業".green());
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
                "submitted" => "已繳".green().to_string(),
                other => other.to_string(),
            }
        };

        table.add_row(vec![
            a.cmid.unwrap_or(a.id).to_string(),
            a.course_shortname.clone(),
            a.name.clone(),
            date_str,
            status,
        ]);
    }

    println!("{table}");

    let label = if all { "作業" } else { "待繳作業" };
    println!("{}", format!("共 {} 項{}", assignments.len(), label).dimmed());

    Ok(())
}

pub async fn submission(
    json: bool,
    base_url: Option<&str>,
    cmid: i64,
    download: bool,
    output_dir: Option<String>,
) -> Result<()> {
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

    // Collect submitted files from plugins
    let submitted_files: Vec<&e3_core::types::FileInfo> = status
        .lastattempt
        .as_ref()
        .and_then(|la| la.submission.as_ref().or(la.teamsubmission.as_ref()))
        .map(|sub| {
            sub.plugins
                .iter()
                .flat_map(|p| &p.fileareas)
                .flat_map(|a| &a.files)
                .collect()
        })
        .unwrap_or_default();

    if download {
        return download_files(json, &client, &submitted_files, output_dir).await;
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
            if !submitted_files.is_empty() {
                println!("\n{}", "已上傳檔案:".bold());
                for f in &submitted_files {
                    println!(
                        "  {} {} ({})",
                        "•".dimmed(),
                        f.filename.as_deref().unwrap_or("?"),
                        output::format_size(f.filesize.unwrap_or(0)),
                    );
                }
                println!(
                    "\n{}",
                    format!("用 e3 submission {cmid} --download 下載檔案").dimmed()
                );
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

async fn download_files(
    json: bool,
    client: &e3_core::MoodleClient,
    file_list: &[&e3_core::types::FileInfo],
    output_dir: Option<String>,
) -> Result<()> {
    if file_list.is_empty() {
        if json {
            output::print_json_success(&serde_json::json!({
                "downloaded": 0,
                "message": "no files to download",
            }));
        } else {
            println!("{}", "沒有已繳交的檔案".yellow());
        }
        return Ok(());
    }

    let dest_dir = PathBuf::from(output_dir.unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dest_dir)
        .map_err(|e| E3Error::Other(format!("Cannot create directory: {e}")))?;

    let mut downloaded = 0;
    for f in file_list {
        let filename = f.filename.as_deref().unwrap_or("unknown");
        let url = match &f.fileurl {
            Some(u) => u,
            None => continue,
        };

        let safe_name = files::sanitize_filename(filename);
        let dest = match files::safe_join(&dest_dir, &safe_name) {
            Some(p) => p,
            None => {
                eprintln!("{} 跳過不安全路徑: {}", "warn:".yellow(), filename);
                continue;
            }
        };

        if !json {
            eprint!("下載 {}...", safe_name);
        }

        match client.download_file(url).await {
            Ok(data) => {
                std::fs::write(&dest, &data)
                    .map_err(|e| E3Error::Other(format!("Write error: {e}")))?;
                downloaded += 1;
                if !json {
                    eprintln!(" {}", "✓".green());
                }
            }
            Err(e) => {
                if !json {
                    eprintln!(" {}", format!("✗ {e}").red());
                }
            }
        }
    }

    if json {
        let names: Vec<_> = file_list
            .iter()
            .filter_map(|f| f.filename.clone())
            .collect();
        output::print_json_success(&serde_json::json!({
            "downloaded": downloaded,
            "files": names,
            "directory": dest_dir.to_string_lossy(),
        }));
    } else {
        println!(
            "\n{} {} 個檔案到 {}",
            "✓ 已下載".green().bold(),
            downloaded,
            dest_dir.display(),
        );
    }

    Ok(())
}

