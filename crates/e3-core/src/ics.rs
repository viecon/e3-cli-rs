use crate::types::ICSEvent;
use chrono::{Datelike, Timelike, Utc};

/// Generate ICS calendar content
pub fn generate_ics(events: &[ICSEvent]) -> String {
    let mut lines = vec![
        "BEGIN:VCALENDAR".into(),
        "VERSION:2.0".into(),
        "PRODID:-//E3 CLI//NYCU E3 Calendar//EN".into(),
        "X-WR-CALNAME:NYCU E3".into(),
        "CALSCALE:GREGORIAN".into(),
        "METHOD:PUBLISH".into(),
        "X-PUBLISHED-TTL:PT6H".into(),
        "REFRESH-INTERVAL;VALUE=DURATION:PT6H".into(),
    ];

    for event in events {
        lines.push("BEGIN:VEVENT".into());
        lines.push(format!("UID:{}", event.uid));
        lines.push(format!("DTSTAMP:{}", format_utc(&Utc::now())));

        if event.all_day {
            lines.push(format!(
                "DTSTART;VALUE=DATE:{}",
                format_date(&event.dtstart)
            ));
            if let Some(end) = &event.dtend {
                lines.push(format!("DTEND;VALUE=DATE:{}", format_date(end)));
            }
        } else {
            lines.push(format!("DTSTART:{}", format_utc(&event.dtstart)));
            if let Some(end) = &event.dtend {
                lines.push(format!("DTEND:{}", format_utc(end)));
            }
        }

        lines.push(format!("SUMMARY:{}", ics_escape(&event.summary)));

        if let Some(desc) = &event.description {
            let clean = strip_html(desc);
            if !clean.is_empty() {
                lines.push(format!("DESCRIPTION:{}", ics_escape(&clean)));
            }
        }

        if let Some(loc) = &event.location {
            lines.push(format!("LOCATION:{}", ics_escape(loc)));
        }

        if !event.categories.is_empty() {
            lines.push(format!("CATEGORIES:{}", event.categories.join(",")));
        }

        // Two alarms: 1 day before + 1 hour before
        lines.push("BEGIN:VALARM".into());
        lines.push("TRIGGER:-P1D".into());
        lines.push("ACTION:DISPLAY".into());
        lines.push("DESCRIPTION:Reminder".into());
        lines.push("END:VALARM".into());

        lines.push("BEGIN:VALARM".into());
        lines.push("TRIGGER:-PT1H".into());
        lines.push("ACTION:DISPLAY".into());
        lines.push("DESCRIPTION:Reminder".into());
        lines.push("END:VALARM".into());

        lines.push("END:VEVENT".into());
    }

    lines.push("END:VCALENDAR".into());
    lines.join("\r\n")
}

fn format_utc(dt: &chrono::DateTime<Utc>) -> String {
    format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second()
    )
}

fn format_date(dt: &chrono::DateTime<Utc>) -> String {
    format!("{:04}{:02}{:02}", dt.year(), dt.month(), dt.day())
}

fn ics_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('\n', "\\n")
        .replace('\r', "")
}

/// Strip HTML tags and decode common entities
pub fn strip_html(html: &str) -> String {
    let re = regex::Regex::new(r"<[^>]+>").unwrap();
    let text = re.replace_all(html, "");
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
