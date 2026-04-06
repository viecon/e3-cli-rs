use crate::config::build_client_with_relogin;
use crate::output;
use colored::Colorize;
use e3_core::error::Result;

pub async fn run(json: bool, base_url: Option<&str>, course: Option<i64>, days: i64) -> Result<()> {
    let (client, _config) = build_client_with_relogin(base_url).await?;

    let sp = if !json {
        Some(output::spinner("取得公告..."))
    } else {
        None
    };

    let courses = if let Some(cid) = course {
        vec![cid]
    } else {
        let c = e3_core::courses::get_enrolled_courses(&client, "inprogress").await?;
        c.iter().map(|c| c.id).collect()
    };

    let forums = e3_core::forums::get_forums(&client, &courses)
        .await
        .unwrap_or_default();

    // Find "news" type forums (announcements)
    let news_forums: Vec<_> = forums
        .iter()
        .filter(|f| f.forum_type.as_deref() == Some("news"))
        .collect();

    let cutoff = chrono::Utc::now().timestamp() - days * 86400;

    // Fetch all forum discussions in parallel
    let discussion_futures: Vec<_> = news_forums
        .iter()
        .map(|forum| {
            let client = &client;
            let course = forum.course;
            let forum_id = forum.id;
            async move {
                let result =
                    e3_core::forums::get_forum_discussions(client, forum_id, -1, 0, 20).await;
                (course, result)
            }
        })
        .collect();

    let results = futures::future::join_all(discussion_futures).await;

    let mut all_posts = Vec::new();
    for (course, result) in results {
        if let Ok(discussions) = result {
            for d in discussions.discussions {
                let modified = d.timemodified.unwrap_or(0);
                if modified >= cutoff {
                    all_posts.push((course, d));
                }
            }
        }
    }

    if let Some(sp) = sp {
        sp.finish_and_clear();
    }

    // Sort by time (newest first)
    all_posts.sort_by(|a, b| {
        b.1.timemodified
            .unwrap_or(0)
            .cmp(&a.1.timemodified.unwrap_or(0))
    });

    if json {
        let items: Vec<_> = all_posts
            .iter()
            .map(|(cid, d)| {
                serde_json::json!({
                    "course_id": cid,
                    "subject": d.subject,
                    "message": d.message,
                    "userfullname": d.userfullname,
                    "timemodified": d.timemodified,
                })
            })
            .collect();
        output::print_json_success(&items);
        return Ok(());
    }

    if all_posts.is_empty() {
        println!("{}", "沒有最近公告".dimmed());
        return Ok(());
    }

    for (_, d) in &all_posts {
        let subject = d.subject.as_deref().unwrap_or("(無標題)");
        let author = d.userfullname.as_deref().unwrap_or("?");
        let time = d.timemodified.map(output::format_date).unwrap_or_default();
        let body = d
            .message
            .as_deref()
            .map(output::strip_html)
            .unwrap_or_default();

        println!("{}", subject.bold());
        println!("  {} • {}", author.dimmed(), time.dimmed());
        if !body.is_empty() {
            // Truncate long messages
            let preview: String = body.chars().take(200).collect();
            println!("  {}", preview.dimmed());
        }
        println!();
    }

    println!(
        "{}",
        format!("共 {} 則公告 (最近 {days} 天)", all_posts.len()).dimmed()
    );

    Ok(())
}
