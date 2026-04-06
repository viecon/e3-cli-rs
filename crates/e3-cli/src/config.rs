use e3_core::client::{AuthMethod, MoodleClient};
use e3_core::error::{E3Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const CONFIG_FILENAME: &str = ".e3rc.json";
const ENV_FILENAME: &str = ".e3.env";
const DEFAULT_BASE_URL: &str = "https://e3p.nycu.edu.tw";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct E3Config {
    pub token: Option<String>,
    pub session: Option<String>,
    // Accept both snake_case and camelCase
    #[serde(alias = "authMode")]
    pub auth_mode: Option<String>,
    pub userid: Option<i64>,
    pub fullname: Option<String>,
    pub sesskey: Option<String>,
    #[serde(alias = "baseUrl")]
    pub base_url: Option<String>,
    #[serde(alias = "vaultPath")]
    pub vault_path: Option<String>,
    #[serde(alias = "excludedCourses", default)]
    pub excluded_courses: Vec<String>,
    #[serde(alias = "excludedExtensions", default)]
    pub excluded_extensions: Vec<String>,
}

impl E3Config {
    pub fn config_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(CONFIG_FILENAME)
    }

    pub fn env_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(ENV_FILENAME)
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            return Err(E3Error::NotAuthenticated);
        }
        let text = std::fs::read_to_string(&path)
            .map_err(|e| E3Error::Other(format!("Cannot read config: {e}")))?;
        serde_json::from_str(&text).map_err(|e| E3Error::Other(format!("Invalid config: {e}")))
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        let text = serde_json::to_string_pretty(self)
            .map_err(|e| E3Error::Other(format!("Serialize error: {e}")))?;
        std::fs::write(&path, &text)
            .map_err(|e| E3Error::Other(format!("Cannot write config: {e}")))?;

        // Set file permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).ok();
        }

        Ok(())
    }

    pub fn delete() -> Result<()> {
        let path = Self::config_path();
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| E3Error::Other(format!("Cannot delete config: {e}")))?;
        }
        Ok(())
    }

    pub fn get_base_url(&self) -> &str {
        self.base_url.as_deref().unwrap_or(DEFAULT_BASE_URL)
    }

    /// Build MoodleClient from config
    pub fn build_client(&self, base_url_override: Option<&str>) -> Result<MoodleClient> {
        let base_url = base_url_override.unwrap_or_else(|| self.get_base_url());

        let auth = match self.auth_mode.as_deref() {
            Some("session") => {
                let cookie = self
                    .session
                    .as_ref()
                    .ok_or(E3Error::NotAuthenticated)?
                    .clone();
                let sesskey = self
                    .sesskey
                    .as_ref()
                    .ok_or(E3Error::NotAuthenticated)?
                    .clone();
                AuthMethod::Session { cookie, sesskey }
            }
            _ => {
                let token = self
                    .token
                    .as_ref()
                    .ok_or(E3Error::NotAuthenticated)?
                    .clone();
                AuthMethod::Token(token)
            }
        };

        MoodleClient::new(Some(base_url), auth)
    }
}

/// Read credentials from ~/.e3.env
pub fn load_env_credentials() -> Option<(String, String)> {
    let path = E3Config::env_path();
    let text = std::fs::read_to_string(&path).ok()?;

    let mut username = None;
    let mut password = None;

    for line in text.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("E3_USERNAME=") {
            username = Some(val.to_string());
        } else if let Some(val) = line.strip_prefix("E3_PASSWORD=") {
            password = Some(val.to_string());
        }
    }

    match (username, password) {
        (Some(u), Some(p)) => Some((u, p)),
        _ => None,
    }
}

/// Save credentials to ~/.e3.env
pub fn save_env_credentials(username: &str, password: &str) -> Result<()> {
    let path = E3Config::env_path();
    let content = format!("E3_USERNAME={username}\nE3_PASSWORD={password}\n");
    std::fs::write(&path, &content)
        .map_err(|e| E3Error::Other(format!("Cannot write env: {e}")))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).ok();
    }

    Ok(())
}

/// Try to re-login using saved credentials
pub async fn try_relogin(config: &mut E3Config) -> Result<String> {
    let (username, password) = load_env_credentials().ok_or(E3Error::NotAuthenticated)?;

    let base_url = config.get_base_url().to_string();
    let token = MoodleClient::login_with_password(Some(&base_url), &username, &password).await?;

    config.token = Some(token.clone());
    config.auth_mode = Some("token".into());
    config.save()?;

    Ok(token)
}

/// Build a client with auto-relogin on SessionExpired.
/// Returns (client, config) so callers can use the refreshed client.
pub async fn build_client_with_relogin(base_url: Option<&str>) -> Result<(MoodleClient, E3Config)> {
    let config = E3Config::load()?;
    let client = config.build_client(base_url)?;

    // Test the connection
    match e3_core::auth::get_site_info(&client).await {
        Ok(_) => Ok((client, config)),
        Err(E3Error::SessionExpired) => {
            // Attempt auto-relogin
            let mut config = config;
            let token = try_relogin(&mut config).await?;
            let client = MoodleClient::new(
                Some(base_url.unwrap_or_else(|| config.get_base_url())),
                e3_core::client::AuthMethod::Token(token),
            )?;
            Ok((client, config))
        }
        Err(e) => Err(e),
    }
}
