use crate::error::{E3Error, Result};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, COOKIE};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;
use url::Url;

const DEFAULT_BASE_URL: &str = "https://e3p.nycu.edu.tw";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
pub enum AuthMethod {
    Token(String),
    Session { cookie: String, sesskey: String },
}

#[derive(Clone)]
pub struct MoodleClient {
    http: reqwest::Client,
    base_url: Url,
    auth: AuthMethod,
}

impl MoodleClient {
    pub fn new(base_url: Option<&str>, auth: AuthMethod) -> Result<Self> {
        let base_url = Url::parse(base_url.unwrap_or(DEFAULT_BASE_URL))
            .map_err(|e| E3Error::Other(format!("Invalid URL: {e}")))?;

        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .cookie_store(true)
            .build()?;

        Ok(Self {
            http,
            base_url,
            auth,
        })
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub fn auth(&self) -> &AuthMethod {
        &self.auth
    }

    pub fn set_auth(&mut self, auth: AuthMethod) {
        self.auth = auth;
    }

    /// Unified API call — REST if token, AJAX if session
    pub async fn call<T: DeserializeOwned>(
        &self,
        func: &str,
        params: &impl Serialize,
    ) -> Result<T> {
        match &self.auth {
            AuthMethod::Token(_) => self.rest_call(func, params).await,
            AuthMethod::Session { .. } => self.ajax_call(func, params).await,
        }
    }

    /// REST API call (token-based)
    async fn rest_call<T: DeserializeOwned>(
        &self,
        func: &str,
        params: &impl Serialize,
    ) -> Result<T> {
        let token = match &self.auth {
            AuthMethod::Token(t) => t,
            _ => return Err(E3Error::NotAuthenticated),
        };

        let mut url = self.base_url.clone();
        url.set_path("/webservice/rest/server.php");
        url.query_pairs_mut()
            .append_pair("wstoken", token)
            .append_pair("moodlewsrestformat", "json")
            .append_pair("wsfunction", func);

        let value = serde_json::to_value(params)
            .map_err(|e| E3Error::Other(format!("Serialize error: {e}")))?;
        let form_params = flatten_params(&value);

        let resp = self
            .http
            .post(url)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(
                form_params
                    .iter()
                    .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                    .collect::<Vec<_>>()
                    .join("&"),
            )
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;

        if status == 301 || status == 302 {
            return Err(E3Error::SessionExpired);
        }

        // Check for Moodle error response
        if let Ok(err) = serde_json::from_str::<MoodleErrorResponse>(&text) {
            if err.exception.is_some() {
                let code = err.errorcode.unwrap_or_default();
                if code == "invalidtoken" || code == "servicerequireslogin" {
                    return Err(E3Error::SessionExpired);
                }
                return Err(E3Error::Api {
                    code,
                    message: err.message.unwrap_or_default(),
                });
            }
        }

        serde_json::from_str(&text).map_err(|e| {
            E3Error::InvalidResponse(format!(
                "JSON parse error: {e}\nBody: {}",
                &text[..text.len().min(500)]
            ))
        })
    }

    /// AJAX API call (session-based)
    async fn ajax_call<T: DeserializeOwned>(
        &self,
        func: &str,
        params: &impl Serialize,
    ) -> Result<T> {
        let (cookie, sesskey) = match &self.auth {
            AuthMethod::Session { cookie, sesskey } => (cookie, sesskey),
            _ => return Err(E3Error::NotAuthenticated),
        };

        let mut url = self.base_url.clone();
        url.set_path("/lib/ajax/service.php");
        url.query_pairs_mut()
            .append_pair("sesskey", sesskey)
            .append_pair("info", func);

        let body = serde_json::json!([{
            "index": 0,
            "methodname": func,
            "args": params,
        }]);

        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            HeaderValue::from_str(&format!("MoodleSession={cookie}"))
                .map_err(|e| E3Error::Other(format!("Invalid cookie: {e}")))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let resp = self
            .http
            .post(url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if status.as_u16() == 403 {
            return Err(E3Error::SessionExpired);
        }

        let text = resp.text().await?;

        let arr: Vec<AjaxResponse> = serde_json::from_str(&text).map_err(|e| {
            if text.contains("loginerrors") || text.contains("login/index.php") {
                return E3Error::SessionExpired;
            }
            E3Error::InvalidResponse(format!("AJAX parse error: {e}"))
        })?;

        let item = arr
            .into_iter()
            .next()
            .ok_or_else(|| E3Error::InvalidResponse("Empty AJAX response".into()))?;

        if item.error.unwrap_or(false) {
            let exc = item.exception.unwrap_or_default();
            let code = exc
                .get("errorcode")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let message = exc
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if code == "servicerequireslogin" {
                return Err(E3Error::SessionExpired);
            }
            return Err(E3Error::Api { code, message });
        }

        let data = item
            .data
            .ok_or_else(|| E3Error::InvalidResponse("AJAX response missing data".into()))?;

        serde_json::from_value(data)
            .map_err(|e| E3Error::InvalidResponse(format!("AJAX data parse error: {e}")))
    }

    /// Fetch an authenticated E3 page, returns HTML body.
    /// For token mode: pluginfile/webservice URLs get token appended;
    /// other pages require a session cookie (call `establish_session` first).
    pub async fn fetch_page(&self, page_url: &str) -> Result<String> {
        let parsed = Url::parse(page_url)
            .map_err(|e| E3Error::Other(format!("Invalid URL: {e}")))?;

        let req = match &self.auth {
            AuthMethod::Token(token) => {
                let path = parsed.path();
                if path.contains("/pluginfile.php") || path.contains("/webservice/") {
                    let mut url = parsed;
                    url.query_pairs_mut().append_pair("token", token);
                    self.http.get(url.as_str())
                } else {
                    // Non-API pages don't accept token param
                    return Err(E3Error::Other(
                        "Token 模式無法存取非 API 頁面，請用 session 或儲存帳密後重試".into(),
                    ));
                }
            }
            AuthMethod::Session { cookie, .. } => {
                self.http
                    .get(page_url)
                    .header(COOKIE, format!("MoodleSession={cookie}"))
            }
        };

        let resp = req.send().await?;
        let status = resp.status();
        if status.as_u16() == 301 || status.as_u16() == 302 || status.as_u16() == 403 {
            return Err(E3Error::SessionExpired);
        }

        let html = resp.text().await?;
        if html.contains("login/index.php") && html.contains("loginerrors") {
            return Err(E3Error::SessionExpired);
        }

        Ok(html)
    }

    /// Establish a Moodle browser session via form login, returns MoodleSession cookie.
    pub async fn establish_session(
        base_url: &Url,
        username: &str,
        password: &str,
    ) -> Result<String> {
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(REQUEST_TIMEOUT)
            .build()?;

        let mut login_url = base_url.clone();
        login_url.set_path("/login/index.php");

        // Step 1: GET the login page to extract logintoken
        let page_resp = http.get(login_url.as_str()).send().await?;

        // Capture initial MoodleSession cookie (pre-login)
        let initial_cookie: Option<String> = page_resp
            .headers()
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .find(|s| s.starts_with("MoodleSession="))
            .and_then(|s| s.split('=').nth(1))
            .and_then(|s| s.split(';').next())
            .map(|s| s.to_string());

        let page = page_resp.text().await?;
        let logintoken = extract_pattern(&page, r#"name="logintoken"\s+value="([^"]+)""#)
            .ok_or_else(|| E3Error::Other("無法取得 logintoken".into()))?;

        // Step 2: POST login form with no-redirect to capture Set-Cookie
        let mut post = http.post(login_url.as_str()).form(&[
            ("username", username),
            ("password", password),
            ("logintoken", logintoken.as_str()),
            ("anchor", ""),
        ]);

        // Send the pre-login cookie if we got one
        if let Some(ref c) = initial_cookie {
            post = post.header(COOKIE, format!("MoodleSession={c}"));
        }

        let resp = post.send().await?;

        // On successful login, Moodle returns 303 with a new MoodleSession cookie
        resp.headers()
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .find(|s| s.starts_with("MoodleSession="))
            .and_then(|s| s.split('=').nth(1))
            .and_then(|s| s.split(';').next())
            .map(|s| s.to_string())
            .ok_or_else(|| E3Error::Other("登入失敗，請確認帳密正確".into()))
    }

    /// Extract sesskey from /my/ page (for session auth)
    pub async fn extract_sesskey(
        &self,
        cookie: &str,
    ) -> Result<(String, Option<i64>, Option<String>)> {
        let mut url = self.base_url.clone();
        url.set_path("/my/");

        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            HeaderValue::from_str(&format!("MoodleSession={cookie}"))
                .map_err(|e| E3Error::Other(format!("Invalid cookie: {e}")))?,
        );

        let resp = self.http.get(url).headers(headers).send().await?;
        let status = resp.status();
        if status.as_u16() == 403 || status.as_u16() == 302 || status.as_u16() == 301 {
            return Err(E3Error::SessionExpired);
        }

        let html = resp.text().await?;

        // Extract sesskey
        let sesskey = extract_pattern(&html, r#""sesskey"\s*:\s*"([^"]+)""#)
            .ok_or_else(|| E3Error::InvalidResponse("Cannot extract sesskey".into()))?;

        // Extract userid
        let userid =
            extract_pattern(&html, r#"data-userid="(\d+)""#).and_then(|s| s.parse::<i64>().ok());

        // Extract fullname (try multiple patterns)
        let fullname = extract_pattern(&html, r#""fullname"\s*:\s*"([^"]+)""#);

        Ok((sesskey, userid, fullname))
    }

    /// Download a file (returns bytes)
    pub async fn download_file(&self, file_url: &str) -> Result<Vec<u8>> {
        let url = match &self.auth {
            AuthMethod::Token(token) => {
                let mut u = Url::parse(file_url)
                    .map_err(|e| E3Error::Other(format!("Invalid file URL: {e}")))?;
                u.query_pairs_mut().append_pair("token", token);
                u.to_string()
            }
            AuthMethod::Session { .. } => file_url.to_string(),
        };

        let mut req = self.http.get(&url);
        if let AuthMethod::Session { cookie, .. } = &self.auth {
            req = req.header(COOKIE, format!("MoodleSession={cookie}"));
        }

        let resp = req.send().await?;
        Ok(resp.bytes().await?.to_vec())
    }

    /// Download a file with streaming progress callback
    pub async fn download_file_streaming(
        &self,
        file_url: &str,
        on_chunk: impl Fn(u64, Option<u64>),
    ) -> Result<Vec<u8>> {
        let url = match &self.auth {
            AuthMethod::Token(token) => {
                let mut u = Url::parse(file_url)
                    .map_err(|e| E3Error::Other(format!("Invalid file URL: {e}")))?;
                u.query_pairs_mut().append_pair("token", token);
                u.to_string()
            }
            AuthMethod::Session { .. } => file_url.to_string(),
        };

        let mut req = self.http.get(&url);
        if let AuthMethod::Session { cookie, .. } = &self.auth {
            req = req.header(COOKIE, format!("MoodleSession={cookie}"));
        }

        let resp = req.send().await?;
        let total = resp.content_length();
        let mut bytes = Vec::new();
        let mut downloaded: u64 = 0;

        let mut stream = resp;
        while let Some(chunk) = stream.chunk().await? {
            downloaded += chunk.len() as u64;
            bytes.extend_from_slice(&chunk);
            on_chunk(downloaded, total);
        }

        Ok(bytes)
    }

    /// Upload a file to Moodle draft area
    pub async fn upload_file(
        &self,
        filename: &str,
        data: Vec<u8>,
        item_id: i64,
    ) -> Result<crate::types::UploadResult> {
        match &self.auth {
            AuthMethod::Token(token) => {
                let mut url = self.base_url.clone();
                url.set_path("/webservice/upload.php");

                let part = reqwest::multipart::Part::bytes(data)
                    .file_name(filename.to_string())
                    .mime_str("application/octet-stream")
                    .map_err(|e| E3Error::Other(format!("MIME error: {e}")))?;

                let form = reqwest::multipart::Form::new()
                    .text("token", token.clone())
                    .text("filearea", "draft")
                    .text("itemid", item_id.to_string())
                    .part("file", part);

                let resp = self.http.post(url).multipart(form).send().await?;
                let text = resp.text().await?;

                // Response is an array with one element
                let results: Vec<crate::types::UploadResult> = serde_json::from_str(&text)
                    .map_err(|e| E3Error::InvalidResponse(format!("Upload parse error: {e}")))?;

                results
                    .into_iter()
                    .next()
                    .ok_or_else(|| E3Error::InvalidResponse("Empty upload response".into()))
            }
            AuthMethod::Session {
                cookie: _,
                sesskey: _,
            } => {
                use base64::Engine;
                let b64 = base64::engine::general_purpose::STANDARD.encode(&data);

                let params = serde_json::json!({
                    "component": "user",
                    "filearea": "draft",
                    "itemid": item_id,
                    "filepath": "/",
                    "filename": filename,
                    "filecontent": b64,
                    "contextlevel": "user",
                    "instanceid": 0
                });

                self.ajax_call("core_files_upload", &params).await
            }
        }
    }

    /// Login with username + password, returns token
    pub async fn login_with_password(
        base_url: Option<&str>,
        username: &str,
        password: &str,
    ) -> Result<String> {
        let base = Url::parse(base_url.unwrap_or(DEFAULT_BASE_URL))
            .map_err(|e| E3Error::Other(format!("Invalid URL: {e}")))?;

        let mut url = base;
        url.set_path("/login/token.php");

        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()?;

        let resp = http
            .post(url)
            .form(&[
                ("username", username),
                ("password", password),
                ("service", "moodle_mobile_app"),
            ])
            .send()
            .await?;

        let result: crate::types::MoodleToken = resp.json().await?;

        if let Some(token) = result.token {
            Ok(token)
        } else {
            let err_msg = result.error.unwrap_or_else(|| "Unknown login error".into());
            Err(E3Error::Api {
                code: "login_failed".into(),
                message: err_msg,
            })
        }
    }
}

// ── Internal types ──

#[derive(Debug, serde::Deserialize)]
struct MoodleErrorResponse {
    pub exception: Option<String>,
    pub errorcode: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct AjaxResponse {
    pub error: Option<bool>,
    pub data: Option<serde_json::Value>,
    pub exception: Option<serde_json::Value>,
}

// ── Parameter flattening ──

/// Flatten nested JSON into bracket notation for REST API
/// e.g. { "courseids": [1, 2] } → [("courseids[0]", "1"), ("courseids[1]", "2")]
pub fn flatten_params(value: &serde_json::Value) -> Vec<(String, String)> {
    let mut result = Vec::new();
    flatten_recursive(value, String::new(), &mut result);
    result
}

fn flatten_recursive(
    value: &serde_json::Value,
    prefix: String,
    result: &mut Vec<(String, String)>,
) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                let new_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}[{key}]")
                };
                flatten_recursive(val, new_prefix, result);
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                let new_prefix = format!("{prefix}[{i}]");
                flatten_recursive(val, new_prefix, result);
            }
        }
        serde_json::Value::String(s) => {
            result.push((prefix, s.clone()));
        }
        serde_json::Value::Number(n) => {
            result.push((prefix, n.to_string()));
        }
        serde_json::Value::Bool(b) => {
            result.push((prefix, if *b { "1" } else { "0" }.into()));
        }
        serde_json::Value::Null => {}
    }
}

fn extract_pattern(text: &str, pattern: &str) -> Option<String> {
    regex::Regex::new(pattern)
        .ok()?
        .captures(text)?
        .get(1)
        .map(|m| m.as_str().to_string())
}

/// URL encoding helper (not using reqwest's, we need direct control)
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut result = String::new();
        for b in s.bytes() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    result.push(b as char);
                }
                _ => {
                    result.push_str(&format!("%{:02X}", b));
                }
            }
        }
        result
    }
}
