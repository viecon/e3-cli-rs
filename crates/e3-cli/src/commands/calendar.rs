use crate::config::build_client_with_relogin;
use crate::output;
use colored::Colorize;
use comfy_table::{presets, ContentArrangement, Table};
use e3_core::error::{E3Error, Result};
use e3_core::types::{CalendarEvent, ICSEvent};

pub async fn run(
    json: bool,
    base_url: Option<&str>,
    days: i64,
    ics: Option<Option<String>>,
    ics_days: i64,
) -> Result<()> {
    let (client, _config) = build_client_with_relogin(base_url).await?;

    let sp = if !json {
        Some(output::spinner("取得行事曆..."))
    } else {
        None
    };

    let events = e3_core::calendar::get_upcoming_events(&client, days).await?;

    if let Some(sp) = sp {
        sp.finish_and_clear();
    }

    // ICS generation
    if let Some(ics_path) = ics {
        let ics_events = e3_core::calendar::get_upcoming_events(&client, ics_days).await?;
        let all_ics_events = calendar_to_ics(&ics_events);
        let ics_content = e3_core::ics::generate_ics(&all_ics_events);
        let path = ics_path.unwrap_or_else(|| "e3-calendar.ics".into());

        std::fs::write(&path, &ics_content)
            .map_err(|e| E3Error::Other(format!("Cannot write ICS: {e}")))?;

        if json {
            output::print_json_success(&serde_json::json!({
                "path": path,
                "events": all_ics_events.len(),
            }));
        } else {
            println!(
                "{} {} ({} 事件)",
                "✓ ICS 已產生:".green().bold(),
                path,
                all_ics_events.len()
            );
        }
        return Ok(());
    }

    if json {
        output::print_json_success(&events);
        return Ok(());
    }

    if events.is_empty() {
        println!("{}", "沒有行事曆事件".dimmed());
        return Ok(());
    }

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL_CONDENSED);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["時間", "事件", "課程", "類型"]);

    for e in &events {
        let time = e.timestart.map(output::format_date).unwrap_or_default();
        let course_name = e
            .course
            .as_ref()
            .and_then(|c| c.shortname.clone())
            .unwrap_or_default();
        let event_type = e
            .modulename
            .clone()
            .unwrap_or_else(|| e.eventtype.clone().unwrap_or_default());
        table.add_row(vec![
            time,
            e.name.clone().unwrap_or_default(),
            course_name,
            event_type,
        ]);
    }

    println!("{table}");
    println!("{}", format!("共 {} 個事件", events.len()).dimmed());

    Ok(())
}

fn calendar_to_ics(events: &[CalendarEvent]) -> Vec<ICSEvent> {
    events
        .iter()
        .filter_map(|e| {
            let timestart = e.timestart?;
            let dtstart = chrono::DateTime::from_timestamp(timestart, 0)?;
            let duration = e.timeduration.unwrap_or(0);

            let module = e.modulename.as_deref().unwrap_or("");
            let uid = if module == "assign" {
                format!("e3-assign-{}@e3p.nycu.edu.tw", e.id)
            } else {
                format!("e3-event-{}@e3p.nycu.edu.tw", e.id)
            };

            let dtend = if duration > 0 {
                chrono::DateTime::from_timestamp(timestart + duration, 0)
            } else if module == "assign" {
                // Assignment: start 1h before deadline
                Some(dtstart)
            } else {
                None
            };

            let actual_start = if module == "assign" && duration == 0 {
                chrono::DateTime::from_timestamp(timestart - 3600, 0).unwrap_or(dtstart)
            } else {
                dtstart
            };

            let summary = e.name.clone().unwrap_or_default();

            let categories = vec![
                e.eventtype.clone().unwrap_or_default(),
                e.course
                    .as_ref()
                    .and_then(|c| c.shortname.clone())
                    .unwrap_or_default(),
            ];

            Some(ICSEvent {
                uid,
                summary,
                description: e.description.clone(),
                dtstart: actual_start,
                dtend,
                location: e.location.clone(),
                categories,
                all_day: false,
            })
        })
        .collect()
}

