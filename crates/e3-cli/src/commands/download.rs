use crate::config::build_client_with_relogin;
use crate::output;
use colored::Colorize;
use e3_core::error::{E3Error, Result};
use e3_core::files;
use std::collections::HashMap;
use std::path::PathBuf;

#[allow(clippy::too_many_arguments)]
pub async fn run(
    json: bool,
    base_url: Option<&str>,
    course: Option<i64>,
    all: bool,
    assignment: Option<i64>,
    type_filter: Option<Vec<String>>,
    output_dir: Option<String>,
    list_only: bool,
    skip_existing: bool,
) -> Result<()> {
    let (client, config) = build_client_with_relogin(base_url).await?;

    // Handle --assignment mode: download assignment intro attachments
    if let Some(cmid) = assignment {
        return download_assignment_attachments(json, &client, cmid, output_dir).await;
    }

    let sp = if !json {
        Some(output::spinner("取得檔案列表..."))
    } else {
        None
    };

    // Determine which courses and get their names
    let courses = if all {
        e3_core::courses::get_enrolled_courses(&client, "inprogress").await?
    } else if let Some(cid) = course {
        // Fetch course info for folder naming
        let all_courses = e3_core::courses::get_enrolled_courses(&client, "all").await?;
        all_courses.into_iter().filter(|c| c.id == cid).collect()
    } else {
        return Err(E3Error::Other("請指定 course ID 或使用 --all".into()));
    };

    // Build course ID → shortname map
    let course_names: HashMap<i64, String> = courses
        .iter()
        .map(|c| {
            let name = c.shortname.clone().unwrap_or_else(|| c.id.to_string());
            (c.id, files::sanitize_filename(&name))
        })
        .collect();

    let course_ids: Vec<i64> = courses.iter().map(|c| c.id).collect();

    let excluded_exts = config.excluded_extensions.clone();

    let type_refs: Option<Vec<&str>> = type_filter
        .as_ref()
        .map(|v| v.iter().map(|s| s.as_str()).collect());

    let mut all_files = Vec::new();
    for &cid in &course_ids {
        match files::list_course_files(&client, cid, type_refs.as_deref()).await {
            Ok(mut f) => {
                // Filter out excluded extensions
                f.retain(|file| {
                    let ext = file
                        .filename
                        .rsplit('.')
                        .next()
                        .unwrap_or("")
                        .to_lowercase();
                    !excluded_exts.iter().any(|e| e.eq_ignore_ascii_case(&ext))
                });
                all_files.extend(f.into_iter().map(|f| (cid, f)));
            }
            Err(e) => {
                if !json {
                    eprintln!("{} course {}: {}", "warn:".yellow(), cid, e);
                }
            }
        }
    }

    if let Some(sp) = sp {
        sp.finish_and_clear();
    }

    if list_only {
        if json {
            let items: Vec<_> = all_files
                .iter()
                .map(|(cid, f)| {
                    serde_json::json!({
                        "course_id": cid,
                        "course_name": course_names.get(cid).unwrap_or(&cid.to_string()),
                        "section": f.section,
                        "module": f.module,
                        "filename": f.filename,
                        "filesize": f.filesize,
                        "fileurl": f.fileurl,
                    })
                })
                .collect();
            output::print_json_success(&items);
        } else {
            for (cid, f) in &all_files {
                let fallback = cid.to_string();
                let cname = course_names.get(cid).unwrap_or(&fallback);
                println!(
                    "  [{}] {}/{} — {} ({})",
                    cname,
                    f.section,
                    f.module,
                    f.filename,
                    output::format_size(f.filesize),
                );
            }
            println!("\n{}", format!("共 {} 個檔案", all_files.len()).dimmed());
        }
        return Ok(());
    }

    // Download with course subfolders
    let base_dir = PathBuf::from(output_dir.unwrap_or_else(|| "e3-downloads".into()));

    let total = all_files.len();
    let mut downloaded = 0;
    let mut skipped = 0;

    for (i, (cid, f)) in all_files.iter().enumerate() {
        let fallback = cid.to_string();
        let course_folder = course_names.get(cid).unwrap_or(&fallback);
        let course_dir = base_dir.join(course_folder);

        // Create course subfolder
        std::fs::create_dir_all(&course_dir)
            .map_err(|e| E3Error::Other(format!("Cannot create directory: {e}")))?;

        let safe_name = files::sanitize_filename(&f.filename);
        let dest = match files::safe_join(&course_dir, &safe_name) {
            Some(p) => p,
            None => {
                eprintln!("{} 跳過不安全路徑: {}", "warn:".yellow(), f.filename);
                continue;
            }
        };

        if skip_existing && dest.exists() {
            skipped += 1;
            continue;
        }

        if !json {
            eprint!("\r[{}/{}] {}/{}...", i + 1, total, course_folder, safe_name);
        }

        match client.download_file(&f.fileurl).await {
            Ok(data) => {
                std::fs::write(&dest, &data)
                    .map_err(|e| E3Error::Other(format!("Write error: {e}")))?;
                downloaded += 1;
            }
            Err(e) => {
                if !json {
                    eprintln!("\n{} {}: {}", "error:".red(), f.filename, e);
                }
            }
        }
    }

    if json {
        output::print_json_success(&serde_json::json!({
            "downloaded": downloaded,
            "skipped": skipped,
            "total": total,
            "directory": base_dir.to_string_lossy(),
        }));
    } else {
        eprintln!(); // clear progress line
        println!(
            "{} {} 檔案 (跳過 {}, 共 {})",
            "✓ 已下載".green().bold(),
            downloaded,
            skipped,
            total,
        );
    }

    Ok(())
}

/// Download assignment intro attachments by cmid
async fn download_assignment_attachments(
    json: bool,
    client: &e3_core::MoodleClient,
    cmid: i64,
    output_dir: Option<String>,
) -> Result<()> {
    let sp = if !json {
        Some(output::spinner("取得作業附件..."))
    } else {
        None
    };

    // Resolve cmid -> (assign_id, course_id)
    let (_assign_id, course_id) =
        e3_core::assignments::resolve_assign_id(client, cmid).await?;

    // Fetch assignment details
    let data = e3_core::assignments::get_assignments(client, &[course_id]).await?;

    // Find the assignment and collect attachments as FileInfo refs
    let assign = data
        .courses
        .iter()
        .flat_map(|c| &c.assignments)
        .find(|a| a.cmid == Some(cmid))
        .ok_or_else(|| E3Error::Other(format!("找不到作業 cmid={cmid}")))?;

    let file_infos: Vec<&e3_core::types::FileInfo> = assign
        .introattachments
        .iter()
        .chain(assign.introfiles.iter())
        .filter(|f| f.fileurl.is_some() && f.filename.is_some())
        .collect();

    if let Some(sp) = sp {
        sp.finish_and_clear();
    }

    if file_infos.is_empty() {
        if json {
            output::print_json_success(&serde_json::json!({
                "downloaded": 0,
                "message": "no attachments",
            }));
        } else {
            println!("{}", "此作業沒有附件".yellow());
        }
        return Ok(());
    }

    let dest_dir = PathBuf::from(output_dir.unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dest_dir)
        .map_err(|e| E3Error::Other(format!("Cannot create directory: {e}")))?;

    let mut downloaded = 0;
    for f in &file_infos {
        let filename = f.filename.as_deref().unwrap_or("unknown");
        let url = f.fileurl.as_deref().unwrap();

        let safe_name = files::sanitize_filename(filename);
        let dest = match files::safe_join(&dest_dir, &safe_name) {
            Some(p) => p,
            None => continue,
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
        let names: Vec<_> = file_infos
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
            "\n{} {} 個附件到 {}",
            "✓ 已下載".green().bold(),
            downloaded,
            dest_dir.display(),
        );
    }

    Ok(())
}

