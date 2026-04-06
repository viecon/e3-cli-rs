use crate::config::E3Config;
use crate::output;
use colored::Colorize;
use e3_core::error::{E3Error, Result};

pub fn run(
    json: bool,
    action: Option<String>,
    key: Option<String>,
    value: Option<String>,
) -> Result<()> {
    match action.as_deref() {
        Some("get") => {
            let config = E3Config::load()?;
            let key = key.ok_or_else(|| E3Error::Other("請指定 key".into()))?;
            let val = get_config_value(&config, &key)?;

            if json {
                output::print_json_success(&serde_json::json!({ &key: val }));
            } else {
                println!("{val}");
            }
        }
        Some("set") => {
            let key = key.ok_or_else(|| E3Error::Other("請指定 key".into()))?;
            let value = value.ok_or_else(|| E3Error::Other("請指定 value".into()))?;
            let mut config = E3Config::load().unwrap_or_default();
            set_config_value(&mut config, &key, &value)?;
            config.save()?;

            if json {
                output::print_json_success(&serde_json::json!({ "key": key, "value": value }));
            } else {
                println!("{} {} = {}", "✓".green(), key.bold(), value);
            }
        }
        Some("list") | None => {
            let config = E3Config::load().unwrap_or_default();

            if json {
                output::print_json_success(&config);
            } else {
                println!(
                    "{}: {}",
                    "token".bold(),
                    mask_token(config.token.as_deref())
                );
                println!(
                    "{}: {}",
                    "auth_mode".bold(),
                    config.auth_mode.as_deref().unwrap_or("—")
                );
                println!(
                    "{}: {}",
                    "userid".bold(),
                    config
                        .userid
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| "—".into())
                );
                println!(
                    "{}: {}",
                    "fullname".bold(),
                    config.fullname.as_deref().unwrap_or("—")
                );
                println!(
                    "{}: {}",
                    "base_url".bold(),
                    config.base_url.as_deref().unwrap_or("—")
                );
                println!(
                    "{}: {}",
                    "vault_path".bold(),
                    config.vault_path.as_deref().unwrap_or("—")
                );
                println!(
                    "{}: {:?}",
                    "excluded_courses".bold(),
                    config.excluded_courses
                );
                println!(
                    "{}: {:?}",
                    "excluded_extensions".bold(),
                    config.excluded_extensions
                );
            }
        }
        Some(other) => {
            return Err(E3Error::Other(format!(
                "Unknown action: {other}. Use get, set, or list"
            )));
        }
    }

    Ok(())
}

fn get_config_value(config: &E3Config, key: &str) -> Result<String> {
    match key {
        "token" => Ok(config.token.clone().unwrap_or_default()),
        "auth_mode" | "authMode" => Ok(config.auth_mode.clone().unwrap_or_default()),
        "userid" => Ok(config.userid.map(|id| id.to_string()).unwrap_or_default()),
        "fullname" => Ok(config.fullname.clone().unwrap_or_default()),
        "base_url" | "baseUrl" => Ok(config.base_url.clone().unwrap_or_default()),
        "vault_path" | "vaultPath" => Ok(config.vault_path.clone().unwrap_or_default()),
        "excluded_courses" | "excludedCourses" => {
            Ok(serde_json::to_string(&config.excluded_courses).unwrap_or_default())
        }
        "excluded_extensions" | "excludedExtensions" => {
            Ok(serde_json::to_string(&config.excluded_extensions).unwrap_or_default())
        }
        _ => Err(E3Error::Other(format!("Unknown key: {key}"))),
    }
}

fn set_config_value(config: &mut E3Config, key: &str, value: &str) -> Result<()> {
    match key {
        "base_url" | "baseUrl" => config.base_url = Some(value.into()),
        "vault_path" | "vaultPath" => config.vault_path = Some(value.into()),
        "excluded_courses" | "excludedCourses" => {
            config.excluded_courses = serde_json::from_str(value)
                .map_err(|e| E3Error::Other(format!("Invalid JSON array: {e}")))?;
        }
        "excluded_extensions" | "excludedExtensions" => {
            config.excluded_extensions = serde_json::from_str(value)
                .map_err(|e| E3Error::Other(format!("Invalid JSON array: {e}")))?;
        }
        _ => return Err(E3Error::Other(format!("Cannot set key: {key}"))),
    }
    Ok(())
}

fn mask_token(token: Option<&str>) -> String {
    match token {
        Some(t) if t.len() > 8 => format!("{}...{}", &t[..4], &t[t.len() - 4..]),
        Some(t) => t.to_string(),
        None => "—".into(),
    }
}
