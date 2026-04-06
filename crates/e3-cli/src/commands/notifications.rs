use crate::config::build_client_with_relogin;
use crate::output;
use colored::Colorize;
use e3_core::error::Result;

pub async fn run(json: bool, base_url: Option<&str>, limit: i32) -> Result<()> {
    let (client, config) = build_client_with_relogin(base_url).await?;
    let userid = config.userid.unwrap_or(0);

    let sp = if !json {
        Some(output::spinner("取得通知..."))
    } else {
        None
    };

    let messages = e3_core::notifications::get_notifications(&client, userid, limit).await?;

    if let Some(sp) = sp {
        sp.finish_and_clear();
    }

    if json {
        output::print_json_success(&messages);
        return Ok(());
    }

    if messages.is_empty() {
        println!("{}", "沒有通知".dimmed());
        return Ok(());
    }

    for m in &messages {
        let is_read = m.timeread.is_some() && m.timeread != Some(0);
        let subject = m.subject.as_deref().unwrap_or("(無標題)");
        let time = m.timecreated.map(output::format_date).unwrap_or_default();
        let from = m.userfromfullname.as_deref().unwrap_or("?");

        let marker = if is_read { " " } else { "●" };
        let marker_colored = if is_read {
            marker.dimmed().to_string()
        } else {
            marker.blue().bold().to_string()
        };

        let subject_display = if is_read {
            subject.dimmed().to_string()
        } else {
            subject.bold().to_string()
        };

        println!(
            "{} {} — {} • {}",
            marker_colored,
            subject_display,
            from.dimmed(),
            time.dimmed()
        );

        // Show preview of message
        if let Some(html) = &m.fullmessagehtml {
            let text = output::strip_html(html);
            if !text.is_empty() {
                let preview: String = text.chars().take(120).collect();
                println!("  {}", preview.dimmed());
            }
        }
    }

    let unread = messages
        .iter()
        .filter(|m| m.timeread.is_none() || m.timeread == Some(0))
        .count();
    println!();
    println!(
        "{}",
        format!("共 {} 則通知 ({unread} 未讀)", messages.len()).dimmed()
    );

    Ok(())
}
