use crate::client::MoodleClient;
use crate::error::Result;
use crate::types::*;

/// Get grade items for a specific course
pub async fn get_course_grades(
    client: &MoodleClient,
    course_id: i64,
    userid: i64,
) -> Result<Vec<GradeItem>> {
    let resp: UserGradeResponse = client
        .call(
            "gradereport_user_get_grade_items",
            &serde_json::json!({
                "courseid": course_id,
                "userid": userid,
            }),
        )
        .await?;

    Ok(resp
        .usergrades
        .into_iter()
        .flat_map(|ug| ug.gradeitems)
        .collect())
}

/// Get grade overview for all courses
pub async fn get_all_grades(client: &MoodleClient, userid: i64) -> Result<OverviewGradesResponse> {
    client
        .call(
            "gradereport_overview_get_course_grades",
            &serde_json::json!({ "userid": userid }),
        )
        .await
}
