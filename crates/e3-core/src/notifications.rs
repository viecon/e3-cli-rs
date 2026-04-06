use crate::client::MoodleClient;
use crate::error::Result;
use crate::types::*;

/// Get user notifications
pub async fn get_notifications(
    client: &MoodleClient,
    userid: i64,
    limit: i32,
) -> Result<Vec<MoodleMessage>> {
    let resp: MessagesResponse = client
        .call(
            "core_message_get_messages",
            &serde_json::json!({
                "useridto": userid,
                "type": "notifications",
                "newestfirst": 1,
                "limitnum": limit,
            }),
        )
        .await?;

    Ok(resp.messages)
}
