use serde::{Deserialize, Deserializer, Serialize};

/// Moodle returns some fields as bool in AJAX and int in REST.
/// This deserializer accepts both.
fn bool_or_int<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<i32>, D::Error> {
    let val = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(val.and_then(|v| match v {
        serde_json::Value::Bool(b) => Some(if b { 1 } else { 0 }),
        serde_json::Value::Number(n) => n.as_i64().map(|n| n as i32),
        _ => None,
    }))
}

// ── Auth ──

#[derive(Debug, Deserialize)]
pub struct MoodleToken {
    pub token: Option<String>,
    pub privatetoken: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SiteInfo {
    pub sitename: Option<String>,
    pub username: Option<String>,
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub fullname: Option<String>,
    pub lang: Option<String>,
    pub userid: Option<i64>,
    pub siteurl: Option<String>,
    pub userpictureurl: Option<String>,
    pub functions: Option<Vec<SiteFunction>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SiteFunction {
    pub name: String,
    pub version: Option<String>,
}

// ── Courses ──

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Course {
    pub id: i64,
    pub shortname: Option<String>,
    pub fullname: Option<String>,
    pub displayname: Option<String>,
    pub enrolledusercount: Option<i64>,
    pub idnumber: Option<String>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub visible: Option<i32>,
    pub summary: Option<String>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub summaryformat: Option<i32>,
    pub format: Option<String>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub showgrades: Option<i32>,
    pub lang: Option<String>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub enablecompletion: Option<i32>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub completionhascriteria: Option<i32>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub completionusertracked: Option<i32>,
    pub category: Option<i64>,
    pub progress: Option<f64>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub completed: Option<i32>,
    pub startdate: Option<i64>,
    pub enddate: Option<i64>,
    pub marker: Option<i64>,
    pub lastaccess: Option<i64>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub isfavourite: Option<i32>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub hidden: Option<i32>,
    #[serde(default)]
    pub overviewfiles: Vec<FileInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimelineCourses {
    pub courses: Vec<Course>,
    pub nextoffset: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CourseSection {
    pub id: i64,
    pub name: Option<String>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub visible: Option<i32>,
    pub summary: Option<String>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub summaryformat: Option<i32>,
    pub section: Option<i64>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub hiddenbynumsections: Option<i32>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub uservisible: Option<i32>,
    #[serde(default)]
    pub modules: Vec<CourseModule>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CourseModule {
    pub id: i64,
    pub url: Option<String>,
    pub name: Option<String>,
    pub instance: Option<i64>,
    pub contextid: Option<i64>,
    pub description: Option<String>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub visible: Option<i32>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub uservisible: Option<i32>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub visibleoncoursepage: Option<i32>,
    pub modicon: Option<String>,
    pub modname: Option<String>,
    pub modplural: Option<String>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub indent: Option<i32>,
    pub onclick: Option<String>,
    pub afterlink: Option<String>,
    pub customdata: Option<String>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub noviewlink: Option<i32>,
    pub completion: Option<i32>,
    #[serde(default)]
    pub contents: Vec<FileInfo>,
}

// ── Files ──

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileInfo {
    #[serde(rename = "type")]
    pub file_type: Option<String>,
    pub filename: Option<String>,
    pub filepath: Option<String>,
    pub filesize: Option<i64>,
    pub fileurl: Option<String>,
    pub timecreated: Option<i64>,
    pub timemodified: Option<i64>,
    pub sortorder: Option<i64>,
    pub mimetype: Option<String>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub isexternalfile: Option<i32>,
    pub userid: Option<i64>,
    pub author: Option<String>,
    pub license: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UploadResult {
    pub itemid: i64,
    pub filename: Option<String>,
}

// ── Assignments ──

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssignmentCourses {
    pub courses: Vec<AssignmentCourse>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssignmentCourse {
    pub id: i64,
    pub fullname: Option<String>,
    pub shortname: Option<String>,
    pub timemodified: Option<i64>,
    #[serde(default)]
    pub assignments: Vec<Assignment>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Assignment {
    pub id: i64,
    pub cmid: Option<i64>,
    pub course: Option<i64>,
    pub name: Option<String>,
    pub nosubmissions: Option<i32>,
    pub submissiondrafts: Option<i32>,
    pub sendnotifications: Option<i32>,
    pub sendlatenotifications: Option<i32>,
    pub sendstudentnotifications: Option<i32>,
    pub duedate: Option<i64>,
    pub allowsubmissionsfromdate: Option<i64>,
    pub grade: Option<f64>,
    pub timemodified: Option<i64>,
    pub completionsubmit: Option<i32>,
    pub cutoffdate: Option<i64>,
    pub gradingduedate: Option<i64>,
    pub teamsubmission: Option<i32>,
    pub requireallteammemberssubmit: Option<i32>,
    pub teamsubmissiongroupingid: Option<i32>,
    pub blindmarking: Option<i32>,
    pub hidegrader: Option<i32>,
    pub revealidentities: Option<i32>,
    pub attemptreopenmethod: Option<String>,
    pub maxattempts: Option<i32>,
    pub markingworkflow: Option<i32>,
    pub markingallocation: Option<i32>,
    pub requiresubmissionstatement: Option<i32>,
    pub preventsubmissionnotingroup: Option<i32>,
    #[serde(default)]
    pub configs: Vec<AssignmentConfig>,
    pub intro: Option<String>,
    pub introformat: Option<i32>,
    #[serde(default)]
    pub introfiles: Vec<FileInfo>,
    #[serde(default)]
    pub introattachments: Vec<FileInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssignmentConfig {
    pub plugin: Option<String>,
    pub subtype: Option<String>,
    pub name: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubmissionStatusResponse {
    pub lastattempt: Option<LastAttempt>,
    pub feedback: Option<SubmissionFeedback>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LastAttempt {
    pub submission: Option<Submission>,
    pub teamsubmission: Option<Submission>,
    #[serde(default)]
    pub submissiongroupmemberswhoneedtosubmit: Vec<serde_json::Value>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub submissionsenabled: Option<i32>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub locked: Option<i32>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub graded: Option<i32>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub canedit: Option<i32>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub caneditowner: Option<i32>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub cansubmit: Option<i32>,
    pub extensionduedate: Option<i64>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub blindmarking: Option<i32>,
    pub gradingstatus: Option<String>,
    #[serde(default)]
    pub usergroups: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Submission {
    pub id: i64,
    pub userid: Option<i64>,
    pub attemptnumber: Option<i32>,
    pub timecreated: Option<i64>,
    pub timemodified: Option<i64>,
    pub status: Option<String>,
    pub groupid: Option<i64>,
    pub assignment: Option<i64>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub latest: Option<i32>,
    #[serde(default)]
    pub plugins: Vec<SubmissionPlugin>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubmissionPlugin {
    #[serde(rename = "type")]
    pub plugin_type: Option<String>,
    pub name: Option<String>,
    #[serde(default)]
    pub fileareas: Vec<PluginFileArea>,
    #[serde(default)]
    pub editorfields: Vec<PluginEditorField>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginFileArea {
    pub area: Option<String>,
    #[serde(default)]
    pub files: Vec<FileInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginEditorField {
    pub name: Option<String>,
    pub description: Option<String>,
    pub text: Option<String>,
    pub format: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubmissionGrade {
    pub userid: Option<i64>,
    pub grade: Option<String>,
    pub grader: Option<i64>,
    pub timemodified: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubmissionFeedback {
    pub grade: Option<SubmissionGrade>,
    pub gradefordisplay: Option<String>,
    pub gradeddate: Option<i64>,
}

/// Processed pending assignment (not raw Moodle type)
#[derive(Debug, Clone, Serialize)]
pub struct PendingAssignment {
    pub id: i64,
    pub cmid: Option<i64>,
    pub course_id: i64,
    pub course_name: String,
    pub course_shortname: String,
    pub name: String,
    pub duedate: Option<i64>,
    pub intro: Option<String>,
    pub submission_status: String,
    pub is_overdue: bool,
    /// Full assignment description HTML (from mod_assign_get_assignments)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Attached files from intro
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<String>,
}

// ── Calendar ──

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CalendarEventsResponse {
    pub events: Vec<CalendarEvent>,
    pub firstid: Option<i64>,
    pub lastid: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CalendarEvent {
    pub id: i64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub descriptionformat: Option<i32>,
    pub location: Option<String>,
    pub categoryid: Option<i64>,
    pub groupid: Option<i64>,
    pub userid: Option<i64>,
    pub repeatid: Option<i64>,
    pub eventcount: Option<i64>,
    pub component: Option<String>,
    pub modulename: Option<String>,
    pub activityname: Option<String>,
    pub activitystr: Option<String>,
    pub instance: Option<i64>,
    pub eventtype: Option<String>,
    pub timestart: Option<i64>,
    pub timeduration: Option<i64>,
    pub timesort: Option<i64>,
    pub timeusermidnight: Option<i64>,
    pub visible: Option<i32>,
    pub timemodified: Option<i64>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub overdue: Option<i32>,
    pub icon: Option<CalendarEventIcon>,
    pub course: Option<CalendarEventCourse>,
    pub url: Option<String>,
    pub action: Option<CalendarEventAction>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CalendarEventIcon {
    pub key: Option<String>,
    pub component: Option<String>,
    pub alttext: Option<String>,
    pub iconurl: Option<String>,
    pub iconclass: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CalendarEventCourse {
    pub id: Option<i64>,
    pub fullname: Option<String>,
    pub shortname: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CalendarEventAction {
    pub name: Option<String>,
    pub url: Option<String>,
    pub itemcount: Option<i64>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub actionable: Option<i32>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub showitemcount: Option<i32>,
}

// ── Grades ──

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserGradeResponse {
    pub usergrades: Vec<UserGradeReport>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserGradeReport {
    pub courseid: Option<i64>,
    pub courseidnumber: Option<String>,
    pub userid: Option<i64>,
    pub userfullname: Option<String>,
    pub useridnumber: Option<String>,
    pub maxdepth: Option<i32>,
    #[serde(default)]
    pub gradeitems: Vec<GradeItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GradeItem {
    pub id: Option<i64>,
    pub itemname: Option<String>,
    pub itemtype: Option<String>,
    pub itemmodule: Option<String>,
    pub iteminstance: Option<i64>,
    pub itemnumber: Option<i64>,
    pub idnumber: Option<String>,
    pub categoryid: Option<i64>,
    pub outcomeid: Option<i64>,
    pub scaleid: Option<i64>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub locked: Option<i32>,
    pub cmid: Option<i64>,
    pub weightraw: Option<f64>,
    pub weightformatted: Option<String>,
    pub graderaw: Option<f64>,
    pub gradedatesubmitted: Option<i64>,
    pub gradedategraded: Option<i64>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub gradehiddenbydate: Option<i32>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub gradeneedsupdate: Option<i32>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub gradeishidden: Option<i32>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub gradeisoverridden: Option<i32>,
    pub gradeformatted: Option<String>,
    pub grademin: Option<f64>,
    pub grademax: Option<f64>,
    pub rangeformatted: Option<String>,
    pub percentageformatted: Option<String>,
    pub feedback: Option<String>,
    pub feedbackformat: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OverviewGradesResponse {
    pub grades: Vec<OverviewGrade>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OverviewGrade {
    pub courseid: Option<i64>,
    pub grade: Option<String>,
    pub rawgrade: Option<String>,
}

// ── Forums ──

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Forum {
    pub id: i64,
    pub course: Option<i64>,
    #[serde(rename = "type")]
    pub forum_type: Option<String>,
    pub name: Option<String>,
    pub intro: Option<String>,
    pub introformat: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ForumDiscussionsResponse {
    pub discussions: Vec<ForumDiscussion>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ForumDiscussion {
    pub id: i64,
    pub name: Option<String>,
    pub groupid: Option<i64>,
    pub timemodified: Option<i64>,
    pub usermodified: Option<i64>,
    pub timestart: Option<i64>,
    pub timeend: Option<i64>,
    pub discussion: Option<i64>,
    pub parent: Option<i64>,
    pub userid: Option<i64>,
    pub created: Option<i64>,
    pub modified: Option<i64>,
    pub mailed: Option<i32>,
    pub subject: Option<String>,
    pub message: Option<String>,
    pub messageformat: Option<i32>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub messagetrust: Option<i32>,
    /// Moodle returns bool or string for this field
    #[serde(default)]
    pub attachment: Option<serde_json::Value>,
    pub totalscore: Option<i32>,
    pub userfullname: Option<String>,
    pub userpictureurl: Option<String>,
    pub numreplies: Option<i32>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub pinned: Option<i32>,
}

// ── Forum Posts ──

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ForumPostsResponse {
    pub posts: Vec<ForumPost>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ForumPost {
    pub id: i64,
    pub discussionid: Option<i64>,
    pub parentid: Option<i64>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub hasparent: Option<i32>,
    pub subject: Option<String>,
    pub message: Option<String>,
    pub messageformat: Option<i32>,
    pub timecreated: Option<i64>,
    pub timemodified: Option<i64>,
    pub author: Option<ForumPostAuthor>,
    #[serde(default)]
    pub attachments: Vec<FileInfo>,
    #[serde(deserialize_with = "bool_or_int", default)]
    pub haswordcount: Option<i32>,
    pub wordcount: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ForumPostAuthor {
    pub id: Option<i64>,
    pub fullname: Option<String>,
}

// ── Notifications ──

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessagesResponse {
    pub messages: Vec<MoodleMessage>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MoodleMessage {
    pub id: i64,
    pub useridfrom: Option<i64>,
    pub useridto: Option<i64>,
    pub subject: Option<String>,
    pub text: Option<String>,
    pub fullmessage: Option<String>,
    pub fullmessageformat: Option<i32>,
    pub fullmessagehtml: Option<String>,
    pub smallmessage: Option<String>,
    pub notification: Option<i32>,
    pub contexturl: Option<String>,
    pub contexturlname: Option<String>,
    pub timecreated: Option<i64>,
    pub timeread: Option<i64>,
    pub usertofullname: Option<String>,
    pub userfromfullname: Option<String>,
}

// ── Course Updates ──

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CourseUpdatesResponse {
    pub instances: Vec<CourseUpdateInstance>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CourseUpdateInstance {
    pub contextlevel: Option<String>,
    pub id: Option<i64>,
    #[serde(default)]
    pub updates: Vec<CourseUpdate>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CourseUpdate {
    pub name: Option<String>,
    pub timeupdated: Option<i64>,
    #[serde(default)]
    pub itemids: Vec<i64>,
}

// ── Course Module Resolution ──

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CourseModuleResponse {
    pub cm: Option<CourseModuleInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CourseModuleInfo {
    pub id: Option<i64>,
    pub instance: Option<i64>,
    pub course: Option<i64>,
}

// ── ICS ──

#[derive(Debug, Clone)]
pub struct ICSEvent {
    pub uid: String,
    pub summary: String,
    pub description: Option<String>,
    pub dtstart: chrono::DateTime<chrono::Utc>,
    pub dtend: Option<chrono::DateTime<chrono::Utc>>,
    pub location: Option<String>,
    pub categories: Vec<String>,
    pub all_day: bool,
}

/// Manual exam event from calendar-events.json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ManualExamEvent {
    pub name: String,
    pub course: String,
    pub date: String,
    #[serde(rename = "startTime")]
    pub start_time: Option<String>,
    #[serde(rename = "endTime")]
    pub end_time: Option<String>,
    pub location: Option<String>,
    #[serde(rename = "allDay")]
    pub all_day: Option<bool>,
}
