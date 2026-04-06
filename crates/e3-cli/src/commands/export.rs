use crate::config::build_client_with_relogin;
use crate::output;
use colored::Colorize;
use e3_core::error::{E3Error, Result};
use std::io::Write;

pub async fn run(
    json: bool,
    base_url: Option<&str>,
    target: &str,
    output_path: Option<String>,
) -> Result<()> {
    match target {
        "grades" => export_grades(json, base_url, output_path).await,
        "assignments" => export_assignments(json, base_url, output_path).await,
        _ => Err(E3Error::Other(format!(
            "Unknown export target: {target}. Use 'grades' or 'assignments'"
        ))),
    }
}

async fn export_grades(
    json: bool,
    base_url: Option<&str>,
    output_path: Option<String>,
) -> Result<()> {
    let (client, config) = build_client_with_relogin(base_url).await?;
    let userid = config.userid.unwrap_or(0);

    let sp = if !json {
        Some(output::spinner("匯出成績..."))
    } else {
        None
    };

    let courses = e3_core::courses::get_enrolled_courses(&client, "inprogress").await?;
    let mut rows: Vec<Vec<String>> = Vec::new();

    for course in &courses {
        let course_name = course.fullname.clone().unwrap_or_default();
        match e3_core::grades::get_course_grades(&client, course.id, userid).await {
            Ok(grades) => {
                for g in grades {
                    if g.itemtype.as_deref() == Some("category") {
                        continue;
                    }
                    rows.push(vec![
                        course_name.clone(),
                        g.itemname.unwrap_or_default(),
                        g.gradeformatted.unwrap_or_else(|| "—".into()),
                        g.rangeformatted.unwrap_or_default(),
                    ]);
                }
            }
            Err(_) => continue,
        }
    }

    if let Some(sp) = sp {
        sp.finish_and_clear();
    }

    let path = output_path.unwrap_or_else(|| "grades.csv".into());
    write_csv(&path, &["課程", "項目", "成績", "範圍"], &rows)?;

    if json {
        output::print_json_success(&serde_json::json!({
            "path": path,
            "rows": rows.len(),
        }));
    } else {
        println!(
            "{} {} ({} 筆)",
            "✓ 已匯出:".green().bold(),
            path,
            rows.len()
        );
    }

    Ok(())
}

async fn export_assignments(
    json: bool,
    base_url: Option<&str>,
    output_path: Option<String>,
) -> Result<()> {
    let (client, _config) = build_client_with_relogin(base_url).await?;

    let sp = if !json {
        Some(output::spinner("匯出作業..."))
    } else {
        None
    };

    let assignments =
        e3_core::assignments::get_pending_assignments_via_calendar(&client, 90).await?;

    if let Some(sp) = sp {
        sp.finish_and_clear();
    }

    let rows: Vec<Vec<String>> = assignments
        .iter()
        .map(|a| {
            vec![
                a.course_name.clone(),
                a.name.clone(),
                a.duedate
                    .map(output::format_date)
                    .unwrap_or_else(|| "無期限".into()),
                if a.is_overdue {
                    "逾期".into()
                } else {
                    a.submission_status.clone()
                },
            ]
        })
        .collect();

    let path = output_path.unwrap_or_else(|| "assignments.csv".into());
    write_csv(&path, &["課程", "作業名稱", "截止日期", "狀態"], &rows)?;

    if json {
        output::print_json_success(&serde_json::json!({
            "path": path,
            "rows": rows.len(),
        }));
    } else {
        println!(
            "{} {} ({} 筆)",
            "✓ 已匯出:".green().bold(),
            path,
            rows.len()
        );
    }

    Ok(())
}

fn write_csv(path: &str, headers: &[&str], rows: &[Vec<String>]) -> Result<()> {
    let mut file = std::fs::File::create(path)
        .map_err(|e| E3Error::Other(format!("Cannot create {path}: {e}")))?;

    // BOM for Excel
    file.write_all(b"\xEF\xBB\xBF")
        .map_err(|e| E3Error::Other(format!("Write error: {e}")))?;

    let mut wtr = csv::Writer::from_writer(file);
    wtr.write_record(headers)
        .map_err(|e| E3Error::Other(format!("CSV error: {e}")))?;

    for row in rows {
        wtr.write_record(row)
            .map_err(|e| E3Error::Other(format!("CSV error: {e}")))?;
    }

    wtr.flush()
        .map_err(|e| E3Error::Other(format!("CSV flush error: {e}")))?;

    Ok(())
}
