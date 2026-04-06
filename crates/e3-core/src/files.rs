use crate::client::MoodleClient;
use crate::error::Result;
use std::collections::HashSet;

/// List all downloadable files in a course.
/// Falls back to HTML scraping if REST API fails (for session-only auth).
pub async fn list_course_files(
    client: &MoodleClient,
    course_id: i64,
    type_filter: Option<&[&str]>,
) -> Result<Vec<CourseFile>> {
    // Try REST API first
    match list_course_files_rest(client, course_id, type_filter).await {
        Ok(files) if !files.is_empty() => Ok(files),
        _ => {
            // Fallback to HTML scraping
            scrape_course_files(client, course_id, type_filter).await
        }
    }
}

/// List files via REST API (core_course_get_contents)
async fn list_course_files_rest(
    client: &MoodleClient,
    course_id: i64,
    type_filter: Option<&[&str]>,
) -> Result<Vec<CourseFile>> {
    let sections = crate::courses::get_course_contents(client, course_id).await?;
    let mut files = Vec::new();

    for section in sections {
        let section_name = section.name.unwrap_or_default();
        for module in section.modules {
            let module_name = module.name.clone().unwrap_or_default();
            for file in module.contents {
                let filename = file.filename.clone().unwrap_or_default();
                let fileurl = file.fileurl.clone().unwrap_or_default();

                if filename.is_empty() || fileurl.is_empty() {
                    continue;
                }

                if !matches_type_filter(&filename, type_filter) {
                    continue;
                }

                files.push(CourseFile {
                    section: section_name.clone(),
                    module: module_name.clone(),
                    filename,
                    fileurl,
                    filesize: file.filesize.unwrap_or(0),
                    timemodified: file.timemodified.unwrap_or(0),
                });
            }
        }
    }

    Ok(files)
}

/// Scrape course page HTML for pluginfile.php links (fallback for session auth)
async fn scrape_course_files(
    client: &MoodleClient,
    course_id: i64,
    type_filter: Option<&[&str]>,
) -> Result<Vec<CourseFile>> {
    let base = client.base_url().clone();
    let course_url = format!("{base}course/view.php?id={course_id}");

    let html = match client.download_file(&course_url).await {
        Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
        Err(_) => return Ok(Vec::new()),
    };

    let mut files = Vec::new();
    let mut seen = HashSet::new();

    // Extract pluginfile.php links
    let re = regex::Regex::new(r#"href="(https?://[^"]*pluginfile\.php/[^"]+)""#).unwrap();
    for cap in re.captures_iter(&html) {
        let url = cap[1].to_string();
        if seen.contains(&url) {
            continue;
        }
        seen.insert(url.clone());

        let filename = match filename_from_url(&url) {
            Some(f) => f,
            None => continue,
        };

        if !matches_type_filter(&filename, type_filter) {
            continue;
        }

        files.push(CourseFile {
            section: String::new(),
            module: String::new(),
            filename,
            fileurl: url,
            filesize: 0,
            timemodified: 0,
        });
    }

    // Also try expanding folder modules
    let folder_re = regex::Regex::new(r#"href="([^"]*mod/folder/view\.php\?id=\d+)""#).unwrap();
    for cap in folder_re.captures_iter(&html) {
        let folder_url = &cap[1];
        if let Ok(folder_bytes) = client.download_file(folder_url).await {
            let folder_html = String::from_utf8_lossy(&folder_bytes);
            for file_cap in re.captures_iter(&folder_html) {
                let url = file_cap[1].to_string();
                if seen.contains(&url) {
                    continue;
                }
                seen.insert(url.clone());

                let filename = match filename_from_url(&url) {
                    Some(f) => f,
                    None => continue,
                };

                if !matches_type_filter(&filename, type_filter) {
                    continue;
                }

                files.push(CourseFile {
                    section: String::new(),
                    module: "folder".into(),
                    filename,
                    fileurl: url,
                    filesize: 0,
                    timemodified: 0,
                });
            }
        }
    }

    Ok(files)
}

/// Extract filename from a URL (last path segment, strip query)
pub(crate) fn filename_from_url(url: &str) -> Option<String> {
    let name = url.split('/').next_back()?.split('?').next()?.to_string();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn matches_type_filter(filename: &str, type_filter: Option<&[&str]>) -> bool {
    match type_filter {
        Some(exts) => {
            let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
            exts.iter().any(|e| e.eq_ignore_ascii_case(&ext))
        }
        None => true,
    }
}

/// Upload multiple files to draft area, returning shared itemid
pub async fn upload_files(client: &MoodleClient, files: Vec<(String, Vec<u8>)>) -> Result<i64> {
    let mut item_id: i64 = 0;

    for (filename, data) in files {
        let result = client.upload_file(&filename, data, item_id).await?;
        item_id = result.itemid;
    }

    Ok(item_id)
}

/// Processed file info for listing
#[derive(Debug, Clone, serde::Serialize)]
pub struct CourseFile {
    pub section: String,
    pub module: String,
    pub filename: String,
    pub fileurl: String,
    pub filesize: i64,
    pub timemodified: i64,
}

/// Sanitize filename for safe local storage (matches TS behavior)
pub fn sanitize_filename(name: &str) -> String {
    // Extract basename — strip directory separators
    let normalized = name.replace('\\', "/");
    let mut s = normalized.rsplit('/').next().unwrap_or(name).to_string();
    // Remove null bytes
    s = s.replace('\0', "");
    // Remove Windows reserved characters
    s = s
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '|' | '?' | '*' => '_',
            _ => c,
        })
        .collect();
    // Remove leading dots
    s = s.trim_start_matches('.').to_string();
    // Fallback for empty filenames
    if s.is_empty() {
        s = "unnamed_file".into();
    }
    s
}

/// Join path safely (prevent traversal)
pub fn safe_join(base: &std::path::Path, child: &str) -> Option<std::path::PathBuf> {
    let sanitized = sanitize_filename(child);
    let joined = base.join(&sanitized);
    // Use canonicalize if base exists, otherwise fall back to lexical check
    let canonical_base = std::fs::canonicalize(base).unwrap_or_else(|_| base.to_path_buf());
    let canonical_joined =
        std::fs::canonicalize(&joined).unwrap_or_else(|_| canonical_base.join(&sanitized));
    if canonical_joined.starts_with(&canonical_base) {
        Some(joined)
    } else {
        None
    }
}
