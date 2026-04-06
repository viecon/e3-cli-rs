use crate::client::MoodleClient;
use crate::error::Result;
use crate::types::*;

/// Get enrolled courses via REST API (token only)
pub async fn get_user_courses(client: &MoodleClient, userid: i64) -> Result<Vec<Course>> {
    client
        .call(
            "core_enrol_get_users_courses",
            &serde_json::json!({ "userid": userid }),
        )
        .await
}

/// Get enrolled courses via timeline classification (AJAX compatible)
pub async fn get_enrolled_courses(
    client: &MoodleClient,
    classification: &str,
) -> Result<Vec<Course>> {
    let resp: TimelineCourses = client
        .call(
            "core_course_get_enrolled_courses_by_timeline_classification",
            &serde_json::json!({
                "classification": classification,
                "limit": 0,
                "offset": 0,
                "sort": "fullname",
            }),
        )
        .await?;
    Ok(resp.courses)
}

/// Get course sections and modules
pub async fn get_course_contents(
    client: &MoodleClient,
    course_id: i64,
) -> Result<Vec<CourseSection>> {
    client
        .call(
            "core_course_get_contents",
            &serde_json::json!({ "courseid": course_id }),
        )
        .await
}

/// Get course updates since a timestamp
pub async fn get_course_updates(
    client: &MoodleClient,
    course_id: i64,
    since: i64,
) -> Result<CourseUpdatesResponse> {
    client
        .call(
            "core_course_get_updates_since",
            &serde_json::json!({
                "courseid": course_id,
                "since": since,
            }),
        )
        .await
}

/// Resolve cmid to assignment instance id
pub async fn get_course_module(client: &MoodleClient, cmid: i64) -> Result<CourseModuleResponse> {
    client
        .call(
            "core_course_get_course_module",
            &serde_json::json!({ "cmid": cmid }),
        )
        .await
}
