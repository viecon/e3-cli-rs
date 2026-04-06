use crate::client::MoodleClient;
use crate::error::{E3Error, Result};
use crate::types::*;

/// Get all assignments for given courses
pub async fn get_assignments(
    client: &MoodleClient,
    course_ids: &[i64],
) -> Result<AssignmentCourses> {
    client
        .call(
            "mod_assign_get_assignments",
            &serde_json::json!({ "courseids": course_ids }),
        )
        .await
}

/// Get submission status for an assignment
pub async fn get_submission_status(
    client: &MoodleClient,
    assign_id: i64,
) -> Result<SubmissionStatusResponse> {
    client
        .call(
            "mod_assign_get_submission_status",
            &serde_json::json!({ "assignid": assign_id }),
        )
        .await
}

/// Submit an assignment with uploaded files
pub async fn save_submission(
    client: &MoodleClient,
    assignment_id: i64,
    draft_item_id: i64,
) -> Result<serde_json::Value> {
    client
        .call(
            "mod_assign_save_submission",
            &serde_json::json!({
                "assignmentid": assignment_id,
                "plugindata": {
                    "files_filemanager": draft_item_id
                }
            }),
        )
        .await
}

/// Resolve cmid → assign instance id
pub async fn resolve_assign_id(client: &MoodleClient, cmid: i64) -> Result<(i64, i64)> {
    let resp = crate::courses::get_course_module(client, cmid).await?;
    let cm = resp
        .cm
        .ok_or_else(|| E3Error::Other(format!("Cannot resolve cmid {cmid}")))?;
    let instance = cm
        .instance
        .ok_or_else(|| E3Error::Other("Missing instance".into()))?;
    let course = cm
        .course
        .ok_or_else(|| E3Error::Other("Missing course".into()))?;
    Ok((instance, course))
}

/// Get pending assignments via calendar events (works with both auth modes)
pub async fn get_pending_assignments_via_calendar(
    client: &MoodleClient,
    days_ahead: i64,
) -> Result<Vec<PendingAssignment>> {
    let now = chrono::Utc::now().timestamp();
    let future = now + days_ahead * 86400;

    let resp: CalendarEventsResponse = client
        .call(
            "core_calendar_get_action_events_by_timesort",
            &serde_json::json!({
                "timesortfrom": now,
                "timesortto": future,
            }),
        )
        .await?;

    let mut assignments = Vec::new();
    for event in resp.events {
        let module = event.modulename.as_deref().unwrap_or("");
        if module != "assign" {
            continue;
        }

        // Only include actionable events (not yet submitted)
        let actionable = event
            .action
            .as_ref()
            .and_then(|a| a.actionable)
            .unwrap_or(0)
            != 0;
        if !actionable {
            continue;
        }

        let instance = match event.instance {
            Some(id) => id,
            None => continue, // skip events without instance
        };

        let course_info = event.course.as_ref();
        let duedate = event.timestart;
        let is_overdue = event.overdue.unwrap_or(0) != 0;

        assignments.push(PendingAssignment {
            id: instance,
            cmid: Some(instance), // calendar API: instance == cmid for assign events
            course_id: course_info.and_then(|c| c.id).unwrap_or(0),
            course_name: course_info
                .and_then(|c| c.fullname.clone())
                .unwrap_or_default(),
            course_shortname: course_info
                .and_then(|c| c.shortname.clone())
                .unwrap_or_default(),
            name: event.name.unwrap_or_default(),
            duedate,
            intro: event.description.clone(),
            submission_status: "new".into(), // actionable means not yet submitted
            is_overdue,
            description: None,
            attachments: Vec::new(),
        });
    }

    Ok(assignments)
}

/// Get ALL assignments for a course (including submitted), via REST API
pub async fn get_all_course_assignments(
    client: &MoodleClient,
    course_ids: &[i64],
) -> Result<Vec<PendingAssignment>> {
    collect_assignments(client, course_ids, false).await
}

/// Get pending assignments via REST API (token only, more accurate status)
pub async fn get_pending_assignments(
    client: &MoodleClient,
    course_ids: &[i64],
) -> Result<Vec<PendingAssignment>> {
    collect_assignments(client, course_ids, true).await
}

/// Shared helper: fetch assignments and their statuses in parallel
async fn collect_assignments(
    client: &MoodleClient,
    course_ids: &[i64],
    pending_only: bool,
) -> Result<Vec<PendingAssignment>> {
    let data = get_assignments(client, course_ids).await?;
    let now = chrono::Utc::now().timestamp();

    // Flatten all valid assignments with course info
    struct AssignInfo {
        assign: Assignment,
        course_id: i64,
        course_name: String,
        course_shortname: String,
    }

    let mut candidates: Vec<AssignInfo> = Vec::new();
    for course in data.courses {
        for assign in course.assignments {
            if assign.nosubmissions.unwrap_or(0) != 0 {
                continue;
            }
            if pending_only {
                let duedate = assign.duedate.unwrap_or(0);
                let cutoff = assign.cutoffdate.unwrap_or(duedate);
                if cutoff > 0 && cutoff < now {
                    continue;
                }
            }
            candidates.push(AssignInfo {
                assign,
                course_id: course.id,
                course_name: course.fullname.clone().unwrap_or_default(),
                course_shortname: course.shortname.clone().unwrap_or_default(),
            });
        }
    }

    // Fetch submission statuses in parallel
    let status_futures: Vec<_> = candidates
        .iter()
        .map(|info| {
            let client = client;
            let assign_id = info.assign.id;
            async move { get_submission_status(client, assign_id).await.ok() }
        })
        .collect();

    let statuses = futures::future::join_all(status_futures).await;

    // Build results
    let mut result = Vec::new();
    for (info, status) in candidates.into_iter().zip(statuses) {
        let sub_status = status
            .as_ref()
            .and_then(|s| s.lastattempt.as_ref())
            .and_then(|la| la.submission.as_ref())
            .and_then(|s| s.status.clone())
            .unwrap_or_else(|| if status.is_some() { "new" } else { "unknown" }.into());

        if pending_only && sub_status == "submitted" {
            continue;
        }

        let duedate = info.assign.duedate.unwrap_or(0);
        let is_overdue = if pending_only {
            duedate > 0 && duedate < now
        } else {
            sub_status != "submitted" && duedate > 0 && duedate < now
        };

        let attachments: Vec<String> = info
            .assign
            .introattachments
            .iter()
            .chain(info.assign.introfiles.iter())
            .filter_map(|f| f.filename.clone())
            .collect();

        result.push(PendingAssignment {
            id: info.assign.id,
            cmid: info.assign.cmid,
            course_id: info.course_id,
            course_name: info.course_name,
            course_shortname: info.course_shortname,
            name: info.assign.name.unwrap_or_default(),
            duedate: if duedate > 0 { Some(duedate) } else { None },
            intro: info.assign.intro.clone(),
            submission_status: sub_status,
            is_overdue,
            description: info.assign.intro,
            attachments,
        });
    }

    Ok(result)
}
