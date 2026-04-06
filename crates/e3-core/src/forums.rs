use crate::client::MoodleClient;
use crate::error::Result;
use crate::types::*;

/// Get forums for given courses
pub async fn get_forums(client: &MoodleClient, course_ids: &[i64]) -> Result<Vec<Forum>> {
    client
        .call(
            "mod_forum_get_forums_by_courses",
            &serde_json::json!({ "courseids": course_ids }),
        )
        .await
}

/// Get discussions in a forum
pub async fn get_forum_discussions(
    client: &MoodleClient,
    forum_id: i64,
    sort_order: i32,
    page: i32,
    per_page: i32,
) -> Result<ForumDiscussionsResponse> {
    client
        .call(
            "mod_forum_get_forum_discussions",
            &serde_json::json!({
                "forumid": forum_id,
                "sortorder": sort_order,
                "page": page,
                "perpage": per_page,
            }),
        )
        .await
}
