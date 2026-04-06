use crate::config::build_client_with_relogin;
use crate::output;
use colored::Colorize;
use e3_core::error::Result;

pub async fn run(json: bool, base_url: Option<&str>, course: Option<i64>, days: i64) -> Result<()> {
    let (client, _config) = build_client_with_relogin(base_url).await?;

    let sp = if !json {
        Some(output::spinner("取得更新..."))
    } else {
        None
    };

    let since = chrono::Utc::now().timestamp() - days * 86400;

    let course_ids = if let Some(cid) = course {
        vec![cid]
    } else {
        let courses = e3_core::courses::get_enrolled_courses(&client, "inprogress").await?;
        courses.iter().map(|c| c.id).collect()
    };

    // Fetch all course updates in parallel
    let update_futures: Vec<_> = course_ids
        .iter()
        .map(|&cid| {
            let client = &client;
            async move {
                let result = e3_core::courses::get_course_updates(client, cid, since).await;
                (cid, result)
            }
        })
        .collect();

    let results = futures::future::join_all(update_futures).await;

    let mut all_updates = Vec::new();
    for (cid, result) in results {
        if let Ok(resp) = result {
            for instance in resp.instances {
                for update in &instance.updates {
                    let time = update.timeupdated.unwrap_or(0);
                    if time >= since {
                        all_updates.push((
                            cid,
                            instance.contextlevel.clone().unwrap_or_default(),
                            update.name.clone().unwrap_or_default(),
                            time,
                            update.itemids.len(),
                        ));
                    }
                }
            }
        }
    }

    if let Some(sp) = sp {
        sp.finish_and_clear();
    }

    // Sort by time (newest first)
    all_updates.sort_by(|a, b| b.3.cmp(&a.3));

    if json {
        let items: Vec<_> = all_updates
            .iter()
            .map(|(cid, ctx, name, time, count)| {
                serde_json::json!({
                    "course_id": cid,
                    "context": ctx,
                    "update_type": name,
                    "time": time,
                    "item_count": count,
                })
            })
            .collect();
        output::print_json_success(&items);
        return Ok(());
    }

    if all_updates.is_empty() {
        println!("{}", "沒有最近更新".dimmed());
        return Ok(());
    }

    for (cid, _ctx, name, time, count) in &all_updates {
        let time_str = output::format_date(*time);
        let update_label = format_update_type(name);
        println!(
            "  {} [{}] {} — {} ({} 項)",
            "•".dimmed(),
            cid,
            update_label,
            time_str.dimmed(),
            count,
        );
    }

    println!(
        "\n{}",
        format!("共 {} 項更新 (最近 {days} 天)", all_updates.len()).dimmed()
    );

    Ok(())
}

fn format_update_type(name: &str) -> String {
    match name {
        "configuration" => "設定變更".into(),
        "fileareas" => "檔案更新".into(),
        "gradeitems" => "成績更新".into(),
        "completion" => "完成度更新".into(),
        "comments" => "評論更新".into(),
        "posts" => "討論更新".into(),
        other => other.into(),
    }
}
