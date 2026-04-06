#[derive(Debug, thiserror::Error)]
pub enum E3Error {
    #[error("Moodle API error ({code}): {message}")]
    Api { code: String, message: String },

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Session expired")]
    SessionExpired,

    #[error("Not authenticated — run `e3 login`")]
    NotAuthenticated,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("{0}")]
    Other(String),
}

/// Structured error info for JSON output
#[derive(serde::Serialize)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
}

impl From<&E3Error> for ErrorInfo {
    fn from(e: &E3Error) -> Self {
        match e {
            E3Error::Api { code, message } => ErrorInfo {
                code: code.clone(),
                message: message.clone(),
            },
            E3Error::SessionExpired => ErrorInfo {
                code: "session_expired".into(),
                message: "登入已過期，請執行 e3 login".into(),
            },
            E3Error::NotAuthenticated => ErrorInfo {
                code: "not_authenticated".into(),
                message: "尚未登入，請執行 e3 login".into(),
            },
            other => ErrorInfo {
                code: "error".into(),
                message: other.to_string(),
            },
        }
    }
}

pub type Result<T> = std::result::Result<T, E3Error>;
