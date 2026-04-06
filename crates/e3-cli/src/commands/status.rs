use crate::config::build_client_with_relogin;
use crate::output;
use colored::Colorize;
use e3_core::error::Result;

pub async fn run(json: bool, base_url: Option<&str>) -> Result<()> {
    let (client, config) = build_client_with_relogin(base_url).await?;
    let userid = config.userid.unwrap_or(0);

    let sp = if !json {
        Some(output::spinner("取得總覽..."))
    } else {
        None
    };

    // Fetch data concurrently
    let (assignments_result, notifications_result, courses_result) = tokio::join!(
        e3_core::assignments::get_pending_assignments_via_calendar(&client, 30),
        e3_core::notifications::get_notifications(&client, userid, 10),
        e3_core::courses::get_enrolled_courses(&client, "inprogress"),
    );

    if let Some(sp) = sp {
        sp.finish_and_clear();
    }

    let assignments = assignments_result.unwrap_or_default();
    let notifications = notifications_result.unwrap_or_default();
    let courses = courses_result.unwrap_or_default();

    let unread_count = notifications
        .iter()
        .filter(|n| n.timeread.is_none() || n.timeread == Some(0))
        .count();

    if json {
        output::print_json_success(&serde_json::json!({
            "assignments": {
                "pending": assignments.len(),
                "items": assignments,
            },
            "notifications": {
                "unread": unread_count,
                "items": notifications,
            },
            "courses": {
                "count": courses.len(),
            },
        }));
        return Ok(());
    }

    // Header
    println!(
        "\n{} {}",
        "E3 總覽".bold().cyan(),
        format!("({})", config.fullname.unwrap_or_default()).dimmed()
    );
    println!("{}", "─".repeat(50));

    // Assignments
    let assign_label = if assignments.is_empty() {
        "✓ 沒有待繳作業".green().to_string()
    } else {
        format!("{} 待繳作業", assignments.len())
            .red()
            .bold()
            .to_string()
    };
    println!("📋 {}", assign_label);

    for a in &assignments {
        let date_str = a
            .duedate
            .map(output::format_date)
            .unwrap_or_else(|| "無期限".into());
        let color = a
            .duedate
            .map(output::urgency_color)
            .unwrap_or(colored::Color::Green);
        let status = if a.is_overdue {
            " [逾期]".red().to_string()
        } else {
            String::new()
        };

        println!(
            "  {} {} — {}{}",
            "•".dimmed(),
            a.name.color(color),
            date_str.dimmed(),
            status,
        );
    }

    // Notifications
    println!();
    if unread_count > 0 {
        println!("🔔 {} 未讀通知", format!("{unread_count}").yellow().bold());
    } else {
        println!("🔔 {}", "沒有未讀通知".green());
    }

    // Courses
    println!("📚 {} 門進行中課程", courses.len());
    println!();

    Ok(())
}
