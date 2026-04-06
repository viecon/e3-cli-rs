use crate::config::build_client_with_relogin;
use crate::output;
use colored::Colorize;
use e3_core::error::{E3Error, Result};

pub async fn run(
    json: bool,
    base_url: Option<&str>,
    cmid: i64,
    file_paths: Vec<String>,
    no_submit: bool,
) -> Result<()> {
    if file_paths.is_empty() {
        return Err(E3Error::Other("請指定要上傳的檔案".into()));
    }

    let (client, _config) = build_client_with_relogin(base_url).await?;

    // Read files
    let mut files = Vec::new();
    for path in &file_paths {
        let data =
            std::fs::read(path).map_err(|e| E3Error::Other(format!("Cannot read {path}: {e}")))?;
        let filename = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path)
            .to_string();
        files.push((filename, data));
    }

    let sp = if !json {
        Some(output::spinner("上傳中..."))
    } else {
        None
    };

    // Upload to draft area
    let item_id = e3_core::files::upload_files(&client, files).await?;

    if !no_submit {
        // Resolve cmid and submit
        let (assign_id, _) = e3_core::assignments::resolve_assign_id(&client, cmid).await?;
        e3_core::assignments::save_submission(&client, assign_id, item_id).await?;
    }

    if let Some(sp) = sp {
        sp.finish_and_clear();
    }

    if json {
        output::print_json_success(&serde_json::json!({
            "item_id": item_id,
            "submitted": !no_submit,
            "files": file_paths,
        }));
    } else {
        let file_list: Vec<_> = file_paths
            .iter()
            .map(|p| {
                std::path::Path::new(p)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(p)
            })
            .collect();

        if no_submit {
            println!(
                "{} {} (未提交，item_id: {})",
                "✓ 已上傳:".green().bold(),
                file_list.join(", "),
                item_id,
            );
        } else {
            println!(
                "{} {} 至作業 {}",
                "✓ 已提交:".green().bold(),
                file_list.join(", "),
                cmid,
            );
        }
    }

    Ok(())
}
