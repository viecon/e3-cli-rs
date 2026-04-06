#[cfg(test)]
mod tests {
    use crate::client::flatten_params;
    use crate::ics::{generate_ics, strip_html};
    use crate::types::ICSEvent;
    use chrono::TimeZone;

    // ── flatten_params ──

    #[test]
    fn flatten_empty_object() {
        let val = serde_json::json!({});
        let result = flatten_params(&val);
        assert!(result.is_empty());
    }

    #[test]
    fn flatten_simple_values() {
        let val = serde_json::json!({
            "userid": 123,
            "name": "test"
        });
        let result = flatten_params(&val);
        assert!(result.contains(&("userid".to_string(), "123".to_string())));
        assert!(result.contains(&("name".to_string(), "test".to_string())));
    }

    #[test]
    fn flatten_array() {
        let val = serde_json::json!({
            "courseids": [1, 2, 3]
        });
        let result = flatten_params(&val);
        assert!(result.contains(&("courseids[0]".to_string(), "1".to_string())));
        assert!(result.contains(&("courseids[1]".to_string(), "2".to_string())));
        assert!(result.contains(&("courseids[2]".to_string(), "3".to_string())));
    }

    #[test]
    fn flatten_nested_object() {
        let val = serde_json::json!({
            "plugindata": {
                "files_filemanager": 42
            }
        });
        let result = flatten_params(&val);
        assert!(result.contains(&(
            "plugindata[files_filemanager]".to_string(),
            "42".to_string()
        )));
    }

    #[test]
    fn flatten_boolean() {
        let val = serde_json::json!({
            "newestfirst": true,
            "disabled": false
        });
        let result = flatten_params(&val);
        assert!(result.contains(&("newestfirst".to_string(), "1".to_string())));
        assert!(result.contains(&("disabled".to_string(), "0".to_string())));
    }

    #[test]
    fn flatten_null_skipped() {
        let val = serde_json::json!({
            "name": "test",
            "other": null
        });
        let result = flatten_params(&val);
        assert_eq!(result.len(), 1);
        assert!(result.contains(&("name".to_string(), "test".to_string())));
    }

    #[test]
    fn flatten_deeply_nested() {
        let val = serde_json::json!({
            "options": {
                "filter": {
                    "ids": [10, 20]
                }
            }
        });
        let result = flatten_params(&val);
        assert!(result.contains(&("options[filter][ids][0]".to_string(), "10".to_string())));
        assert!(result.contains(&("options[filter][ids][1]".to_string(), "20".to_string())));
    }

    #[test]
    fn flatten_empty_array() {
        let val = serde_json::json!({ "ids": [] });
        let result = flatten_params(&val);
        assert!(result.is_empty());
    }

    #[test]
    fn flatten_array_of_objects() {
        let val = serde_json::json!({
            "items": [
                { "id": 1, "name": "a" },
                { "id": 2, "name": "b" }
            ]
        });
        let result = flatten_params(&val);
        assert!(result.contains(&("items[0][id]".to_string(), "1".to_string())));
        assert!(result.contains(&("items[0][name]".to_string(), "a".to_string())));
        assert!(result.contains(&("items[1][id]".to_string(), "2".to_string())));
        assert!(result.contains(&("items[1][name]".to_string(), "b".to_string())));
    }

    // ── filename_from_url ──

    #[test]
    fn filename_from_url_basic() {
        use crate::files::filename_from_url;
        assert_eq!(
            filename_from_url(
                "https://e3.nycu.edu.tw/pluginfile.php/123/mod_resource/content/0/lecture.pdf"
            ),
            Some("lecture.pdf".into())
        );
    }

    #[test]
    fn filename_from_url_with_query() {
        use crate::files::filename_from_url;
        assert_eq!(
            filename_from_url("https://e3.nycu.edu.tw/pluginfile.php/123/file.pdf?forcedownload=1"),
            Some("file.pdf".into())
        );
    }

    #[test]
    fn filename_from_url_trailing_slash() {
        use crate::files::filename_from_url;
        // Trailing slash → empty segment → None
        assert_eq!(filename_from_url("https://example.com/"), None);
    }

    // ── strip_html ──

    #[test]
    fn strip_html_basic() {
        assert_eq!(strip_html("<p>Hello</p>"), "Hello");
        assert_eq!(strip_html("<b>bold</b> text"), "bold text");
    }

    #[test]
    fn strip_html_entities() {
        assert_eq!(strip_html("a &amp; b"), "a & b");
        assert_eq!(strip_html("&lt;tag&gt;"), "<tag>");
        assert_eq!(strip_html("&quot;quoted&quot;"), "\"quoted\"");
    }

    #[test]
    fn strip_html_whitespace() {
        assert_eq!(strip_html("  hello   world  "), "hello world");
        assert_eq!(strip_html("<p>line1</p>\n<p>line2</p>"), "line1 line2");
    }

    #[test]
    fn strip_html_empty() {
        assert_eq!(strip_html(""), "");
        assert_eq!(strip_html("<br/>"), "");
    }

    // ── ICS generation ──

    #[test]
    fn ics_basic_event() {
        let events = vec![ICSEvent {
            uid: "test-1@e3".into(),
            summary: "Test Event".into(),
            description: None,
            dtstart: chrono::Utc.with_ymd_and_hms(2026, 4, 16, 5, 0, 0).unwrap(),
            dtend: Some(chrono::Utc.with_ymd_and_hms(2026, 4, 16, 7, 0, 0).unwrap()),
            location: None,
            categories: vec![],
            all_day: false,
        }];
        let ics = generate_ics(&events);

        assert!(ics.contains("BEGIN:VCALENDAR"));
        assert!(ics.contains("END:VCALENDAR"));
        assert!(ics.contains("UID:test-1@e3"));
        assert!(ics.contains("SUMMARY:Test Event"));
        assert!(ics.contains("DTSTART:20260416T050000Z"));
        assert!(ics.contains("DTEND:20260416T070000Z"));
        assert!(ics.contains("PRODID:-//E3 CLI//NYCU E3 Calendar//EN"));
        assert!(ics.contains("REFRESH-INTERVAL;VALUE=DURATION:PT6H"));
    }

    #[test]
    fn ics_all_day_event() {
        let events = vec![ICSEvent {
            uid: "test-allday@e3".into(),
            summary: "All Day".into(),
            description: None,
            dtstart: chrono::Utc.with_ymd_and_hms(2026, 4, 16, 0, 0, 0).unwrap(),
            dtend: None,
            location: None,
            categories: vec![],
            all_day: true,
        }];
        let ics = generate_ics(&events);
        assert!(ics.contains("DTSTART;VALUE=DATE:20260416"));
        assert!(!ics.contains("DTSTART:2026"));
    }

    #[test]
    fn ics_two_alarms() {
        let events = vec![ICSEvent {
            uid: "test-alarm@e3".into(),
            summary: "Alarm Test".into(),
            description: None,
            dtstart: chrono::Utc.with_ymd_and_hms(2026, 4, 16, 5, 0, 0).unwrap(),
            dtend: None,
            location: None,
            categories: vec![],
            all_day: false,
        }];
        let ics = generate_ics(&events);

        let alarm_count = ics.matches("BEGIN:VALARM").count();
        assert_eq!(alarm_count, 2);
        assert!(ics.contains("TRIGGER:-P1D"));
        assert!(ics.contains("TRIGGER:-PT1H"));
    }

    #[test]
    fn ics_escape_special_chars() {
        let events = vec![ICSEvent {
            uid: "test-escape@e3".into(),
            summary: "Test; with, special\\chars".into(),
            description: Some("<p>Hello</p> <p>World</p>".into()),
            dtstart: chrono::Utc.with_ymd_and_hms(2026, 4, 16, 5, 0, 0).unwrap(),
            dtend: None,
            location: None,
            categories: vec![],
            all_day: false,
        }];
        let ics = generate_ics(&events);
        assert!(ics.contains("SUMMARY:Test\\; with\\, special\\\\chars"));
        // Description is strip_html'd then escaped
        assert!(ics.contains("DESCRIPTION:Hello World"));
    }

    // ── File sanitization ──

    #[test]
    fn sanitize_directory_traversal() {
        use crate::files::sanitize_filename;
        // basename extraction: "../../../etc/passwd" → "passwd"
        assert_eq!(sanitize_filename("../../../etc/passwd"), "passwd");
        assert_eq!(sanitize_filename("..\\..\\windows\\system32"), "system32");
    }

    #[test]
    fn sanitize_strips_path() {
        use crate::files::sanitize_filename;
        assert_eq!(sanitize_filename("path/to/file.pdf"), "file.pdf");
        assert_eq!(sanitize_filename("path\\to\\file.pdf"), "file.pdf");
    }

    #[test]
    fn sanitize_leading_dots() {
        use crate::files::sanitize_filename;
        assert_eq!(sanitize_filename("...hidden"), "hidden");
        assert_eq!(sanitize_filename(".gitignore"), "gitignore");
    }

    #[test]
    fn sanitize_empty_becomes_unnamed() {
        use crate::files::sanitize_filename;
        assert_eq!(sanitize_filename(""), "unnamed_file");
        assert_eq!(sanitize_filename("..."), "unnamed_file");
    }

    #[test]
    fn sanitize_dangerous_chars() {
        use crate::files::sanitize_filename;
        let result = sanitize_filename("file<>:\"|?*.pdf");
        assert!(!result.contains('<'));
        assert!(!result.contains('>'));
        assert!(!result.contains(':'));
        assert!(!result.contains('"'));
        assert!(!result.contains('|'));
        assert!(!result.contains('?'));
        assert!(!result.contains('*'));
        assert!(result.ends_with(".pdf"));
    }

    #[test]
    fn sanitize_null_bytes() {
        use crate::files::sanitize_filename;
        let result = sanitize_filename("file\x00.pdf");
        assert!(!result.contains('\0'));
        assert!(result.contains("file"));
    }

    #[test]
    fn sanitize_normal_filename_preserved() {
        use crate::files::sanitize_filename;
        assert_eq!(sanitize_filename("lecture-01.pdf"), "lecture-01.pdf");
        assert_eq!(sanitize_filename("日本語テスト.pdf"), "日本語テスト.pdf");
    }

    #[test]
    fn safe_join_basic() {
        use crate::files::safe_join;
        use std::path::Path;
        let result = safe_join(Path::new("/tmp/output"), "file.pdf");
        assert!(result.is_some());
    }

    #[test]
    fn safe_join_rejects_traversal() {
        use crate::files::safe_join;
        use std::path::Path;
        // After sanitize_filename, ".." becomes "" so it stays in base
        let result = safe_join(Path::new("/tmp/output"), "../../../etc/passwd");
        // The sanitized result should still be within base
        if let Some(path) = &result {
            assert!(path.starts_with("/tmp/output"));
        }
    }

    // ── MoodleClient ──

    #[test]
    fn client_requires_valid_url() {
        use crate::client::{AuthMethod, MoodleClient};
        let result = MoodleClient::new(Some("not-a-url"), AuthMethod::Token("test".into()));
        assert!(result.is_err());
    }

    #[test]
    fn client_creates_with_token() {
        use crate::client::{AuthMethod, MoodleClient};
        let client = MoodleClient::new(None, AuthMethod::Token("test-token".into())).unwrap();
        assert_eq!(client.base_url().as_str(), "https://e3p.nycu.edu.tw/");
    }

    #[test]
    fn client_creates_with_custom_url() {
        use crate::client::{AuthMethod, MoodleClient};
        let client = MoodleClient::new(
            Some("https://example.com"),
            AuthMethod::Token("test".into()),
        )
        .unwrap();
        assert_eq!(client.base_url().as_str(), "https://example.com/");
    }

    #[test]
    fn client_creates_with_session() {
        use crate::client::{AuthMethod, MoodleClient};
        let client = MoodleClient::new(
            None,
            AuthMethod::Session {
                cookie: "abc".into(),
                sesskey: "xyz".into(),
            },
        )
        .unwrap();
        assert_eq!(client.base_url().as_str(), "https://e3p.nycu.edu.tw/");
    }

    // ── Error types ──

    #[test]
    fn error_info_from_api_error() {
        use crate::error::{E3Error, ErrorInfo};
        let err = E3Error::Api {
            code: "invalidtoken".into(),
            message: "Token expired".into(),
        };
        let info = ErrorInfo::from(&err);
        assert_eq!(info.code, "invalidtoken");
        assert_eq!(info.message, "Token expired");
    }

    #[test]
    fn error_info_from_session_expired() {
        use crate::error::{E3Error, ErrorInfo};
        let err = E3Error::SessionExpired;
        let info = ErrorInfo::from(&err);
        assert_eq!(info.code, "session_expired");
    }

    #[test]
    fn error_info_from_not_authenticated() {
        use crate::error::{E3Error, ErrorInfo};
        let err = E3Error::NotAuthenticated;
        let info = ErrorInfo::from(&err);
        assert_eq!(info.code, "not_authenticated");
    }
}
