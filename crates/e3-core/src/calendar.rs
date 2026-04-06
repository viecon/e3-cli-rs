use crate::client::MoodleClient;
use crate::error::Result;
use crate::types::*;

/// Get upcoming calendar events for the next N days
pub async fn get_upcoming_events(client: &MoodleClient, days: i64) -> Result<Vec<CalendarEvent>> {
    let now = chrono::Utc::now().timestamp();
    let past = now - 86400; // include events from past 24h (matching TS behavior)
    let future = now + days * 86400;

    let resp: CalendarEventsResponse = client
        .call(
            "core_calendar_get_action_events_by_timesort",
            &serde_json::json!({
                "timesortfrom": past,
                "timesortto": future,
            }),
        )
        .await?;

    Ok(resp.events)
}
