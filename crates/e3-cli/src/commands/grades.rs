use crate::config::build_client_with_relogin;
use crate::output;
use colored::Colorize;
use comfy_table::{presets, ContentArrangement, Table};
use e3_core::error::Result;

pub async fn run(json: bool, base_url: Option<&str>, course: Option<i64>) -> Result<()> {
    let (client, config) = build_client_with_relogin(base_url).await?;
    let userid = config.userid.unwrap_or(0);

    let sp = if !json {
        Some(output::spinner("取得成績..."))
    } else {
        None
    };

    if let Some(course_id) = course {
        // Single course grades
        let grades = e3_core::grades::get_course_grades(&client, course_id, userid).await?;

        if let Some(sp) = sp {
            sp.finish_and_clear();
        }

        if json {
            output::print_json_success(&grades);
            return Ok(());
        }

        if grades.is_empty() {
            println!("{}", "沒有成績資料".dimmed());
            return Ok(());
        }

        let mut table = Table::new();
        table.load_preset(presets::UTF8_FULL_CONDENSED);
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec!["項目", "成績", "百分比", "範圍"]);

        for g in &grades {
            // Skip category items
            if g.itemtype.as_deref() == Some("category") {
                continue;
            }

            table.add_row(vec![
                g.itemname.clone().unwrap_or_else(|| "—".into()),
                g.gradeformatted.clone().unwrap_or_else(|| "—".into()),
                g.percentageformatted.clone().unwrap_or_else(|| "—".into()),
                g.rangeformatted.clone().unwrap_or_else(|| "—".into()),
            ]);
        }

        println!("{table}");
    } else {
        // All courses overview
        let overview = e3_core::grades::get_all_grades(&client, userid).await?;
        let courses = e3_core::courses::get_enrolled_courses(&client, "inprogress").await?;

        if let Some(sp) = sp {
            sp.finish_and_clear();
        }

        if json {
            output::print_json_success(&serde_json::json!({
                "grades": overview.grades,
                "courses": courses,
            }));
            return Ok(());
        }

        let mut table = Table::new();
        table.load_preset(presets::UTF8_FULL_CONDENSED);
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec!["課程", "成績"]);

        for g in &overview.grades {
            let course_name = courses
                .iter()
                .find(|c| Some(c.id) == g.courseid)
                .and_then(|c| c.shortname.clone())
                .unwrap_or_else(|| g.courseid.map(|id| id.to_string()).unwrap_or_default());

            table.add_row(vec![
                course_name,
                g.grade.clone().unwrap_or_else(|| "—".into()),
            ]);
        }

        println!("{table}");
    }

    Ok(())
}
