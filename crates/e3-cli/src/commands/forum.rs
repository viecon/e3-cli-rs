use crate::config::build_client_with_relogin;
use crate::output;
use colored::Colorize;
use e3_core::error::Result;

/// List forums in a course, or show discussions in a specific forum
pub async fn run(
    json: bool,
    base_url: Option<&str>,
    target: i64,
    thread: Option<i64>,
    search: Option<String>,
) -> Result<()> {
    let (client, _config) = build_client_with_relogin(base_url).await?;

    // If --thread is given, show that discussion's posts
    if let Some(discussion_id) = thread {
        return show_thread(json, &client, discussion_id).await;
    }

    // If --search is given, search across all posts
    if let Some(keyword) = search {
        return search_forums(json, &client, target, &keyword).await;
    }

    let sp = if !json {
        Some(output::spinner("取得討論區..."))
    } else {
        None
    };

    // Try as forum_id first: fetch discussions directly
    let discussions_result =
        e3_core::forums::get_forum_discussions(&client, target, -1, 0, 50).await;

    match discussions_result {
        Ok(resp) => {
            if let Some(sp) = sp {
                sp.finish_and_clear();
            }
            show_discussions(json, target, &resp.discussions)
        }
        Err(_) => {
            // Fallback: treat target as course_id, list all forums
            let forums = e3_core::forums::get_forums(&client, &[target]).await?;

            if forums.is_empty() {
                if let Some(sp) = sp {
                    sp.finish_and_clear();
                }
                if json {
                    output::print_json_success(&Vec::<()>::new());
                } else {
                    println!("{}", "此課程沒有討論區".dimmed());
                }
                return Ok(());
            }

            // Fetch discussions for all forums in parallel
            let disc_futures: Vec<_> = forums
                .iter()
                .map(|forum| {
                    let client = &client;
                    let forum_id = forum.id;
                    let forum_name = forum.name.clone();
                    async move {
                        let result =
                            e3_core::forums::get_forum_discussions(client, forum_id, -1, 0, 20)
                                .await;
                        (forum_id, forum_name, result)
                    }
                })
                .collect();

            let results = futures::future::join_all(disc_futures).await;

            if let Some(sp) = sp {
                sp.finish_and_clear();
            }

            if json {
                let items: Vec<_> = results
                    .iter()
                    .map(|(fid, fname, result)| {
                        let discussions = result
                            .as_ref()
                            .map(|r| &r.discussions[..])
                            .unwrap_or(&[]);
                        serde_json::json!({
                            "forum_id": fid,
                            "forum_name": fname,
                            "discussions": discussions,
                        })
                    })
                    .collect();
                output::print_json_success(&items);
                return Ok(());
            }

            for (forum_id, forum_name, result) in &results {
                let name = forum_name.as_deref().unwrap_or("?");
                println!("{} (id: {})", name.bold().cyan(), forum_id);

                if let Ok(resp) = result {
                    if resp.discussions.is_empty() {
                        println!("  {}", "(空)".dimmed());
                    }
                    for d in &resp.discussions {
                        print_discussion_line(d);
                    }
                } else {
                    println!("  {}", "無法取得討論".red());
                }
                println!();
            }

            Ok(())
        }
    }
}

fn print_discussion_line(d: &e3_core::types::ForumDiscussion) {
    let subject = d.subject.as_deref().unwrap_or("(無標題)");
    let author = d.userfullname.as_deref().unwrap_or("?");
    let time = d
        .timemodified
        .map(output::format_date)
        .unwrap_or_default();
    let replies = d.numreplies.unwrap_or(0);

    let replies_str = if replies > 0 {
        format!(" ({replies} replies)")
    } else {
        String::new()
    };

    println!(
        "  [{}] {}: {}{}",
        time.dimmed(),
        author.dimmed(),
        subject,
        replies_str.dimmed(),
    );
}

fn show_discussions(
    json: bool,
    forum_id: i64,
    discussions: &[e3_core::types::ForumDiscussion],
) -> Result<()> {
    if json {
        output::print_json_success(&serde_json::json!({
            "forum_id": forum_id,
            "discussions": discussions,
        }));
        return Ok(());
    }

    if discussions.is_empty() {
        println!("{}", "沒有討論".dimmed());
        return Ok(());
    }

    for d in discussions {
        print_discussion_line(d);
    }

    println!(
        "\n{}",
        format!(
            "共 {} 則討論 (用 --thread <id> 查看完整內容)",
            discussions.len()
        )
        .dimmed()
    );

    Ok(())
}

async fn show_thread(
    json: bool,
    client: &e3_core::MoodleClient,
    discussion_id: i64,
) -> Result<()> {
    let sp = if !json {
        Some(output::spinner("取得討論串..."))
    } else {
        None
    };

    let resp = e3_core::forums::get_discussion_posts(client, discussion_id).await?;

    if let Some(sp) = sp {
        sp.finish_and_clear();
    }

    if json {
        output::print_json_success(&resp);
        return Ok(());
    }

    if resp.posts.is_empty() {
        println!("{}", "沒有貼文".dimmed());
        return Ok(());
    }

    for post in &resp.posts {
        print_post(post, false);
    }

    Ok(())
}

async fn search_forums(
    json: bool,
    client: &e3_core::MoodleClient,
    target: i64,
    keyword: &str,
) -> Result<()> {
    let sp = if !json {
        Some(output::spinner(&format!("搜尋 \"{keyword}\"...")))
    } else {
        None
    };

    // Collect forum IDs: try as forum_id first, fallback to course_id
    let forum_ids: Vec<i64> =
        match e3_core::forums::get_forum_discussions(client, target, -1, 0, 1).await {
            Ok(_) => vec![target],
            Err(_) => {
                let forums = e3_core::forums::get_forums(client, &[target]).await?;
                forums.iter().map(|f| f.id).collect()
            }
        };

    // Get all discussions from all forums
    let disc_futures: Vec<_> = forum_ids
        .iter()
        .map(|&fid| {
            let client = client;
            async move {
                e3_core::forums::get_forum_discussions(client, fid, -1, 0, 100)
                    .await
                    .ok()
            }
        })
        .collect();

    let disc_results = futures::future::join_all(disc_futures).await;
    let discussion_ids: Vec<i64> = disc_results
        .into_iter()
        .flatten()
        .flat_map(|r| r.discussions)
        .map(|d| d.id)
        .collect();

    // Get posts for each discussion
    let post_futures: Vec<_> = discussion_ids
        .iter()
        .map(|&did| {
            let client = client;
            async move { e3_core::forums::get_discussion_posts(client, did).await.ok() }
        })
        .collect();

    let post_results = futures::future::join_all(post_futures).await;

    // Filter posts containing keyword (case-insensitive)
    let keyword_lower = keyword.to_lowercase();
    let mut matches = Vec::new();

    for resp in post_results.into_iter().flatten() {
        for post in resp.posts {
            let subject_match = post
                .subject
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(&keyword_lower);
            let body_match = post
                .message
                .as_deref()
                .map(output::strip_html)
                .unwrap_or_default()
                .to_lowercase()
                .contains(&keyword_lower);

            if subject_match || body_match {
                matches.push(post);
            }
        }
    }

    if let Some(sp) = sp {
        sp.finish_and_clear();
    }

    if json {
        output::print_json_success(&serde_json::json!({
            "keyword": keyword,
            "matches": matches.len(),
            "posts": matches,
        }));
        return Ok(());
    }

    if matches.is_empty() {
        println!("{}", format!("找不到包含 \"{keyword}\" 的貼文").dimmed());
        return Ok(());
    }

    for post in &matches {
        print_post(post, true);
    }

    println!(
        "\n{}",
        format!("共 {} 則貼文包含 \"{keyword}\"", matches.len()).dimmed()
    );

    Ok(())
}

fn print_post(post: &e3_core::types::ForumPost, highlight: bool) {
    let author = post
        .author
        .as_ref()
        .and_then(|a| a.fullname.as_deref())
        .unwrap_or("?");
    let subject = post.subject.as_deref().unwrap_or("");
    let time = post
        .timecreated
        .map(output::format_date)
        .unwrap_or_default();
    let body = post
        .message
        .as_deref()
        .map(output::strip_html)
        .unwrap_or_default();

    let is_reply = post.parentid.unwrap_or(0) != 0;
    let indent = if is_reply { "  " } else { "" };

    if !subject.is_empty() {
        println!("{}{}", indent, subject.bold());
    }

    let disc_hint = if highlight {
        format!(" [discussion {}]", post.discussionid.unwrap_or(0))
    } else {
        String::new()
    };

    println!(
        "{}{} • {}{}",
        indent,
        author.dimmed(),
        time.dimmed(),
        disc_hint.dimmed()
    );
    if !body.trim().is_empty() {
        for line in body.lines() {
            println!("{}{}", indent, line);
        }
    }

    // Show attachments
    for att in &post.attachments {
        if let Some(filename) = &att.filename {
            println!("{}  {} {}", indent, "\u{1f4ce}".dimmed(), filename);
        }
    }

    println!();
}
