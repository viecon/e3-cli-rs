use crate::config::{save_env_credentials, E3Config};
use crate::output;
use colored::Colorize;
use e3_core::client::{AuthMethod, MoodleClient};
use e3_core::error::{E3Error, Result};

pub async fn login(
    json: bool,
    base_url: Option<&str>,
    username: Option<String>,
    password: Option<String>,
    token: Option<String>,
    session: Option<String>,
) -> Result<()> {
    let effective_base = base_url.unwrap_or("https://e3p.nycu.edu.tw");

    if let Some(token_val) = token {
        // Token-based login
        let sp = if !json {
            Some(output::spinner("驗證 token..."))
        } else {
            None
        };

        let client = MoodleClient::new(Some(effective_base), AuthMethod::Token(token_val.clone()))?;
        let info = e3_core::auth::get_site_info(&client).await?;

        if let Some(sp) = sp {
            sp.finish_and_clear();
        }

        let config = E3Config {
            token: Some(token_val),
            auth_mode: Some("token".into()),
            userid: info.userid,
            fullname: info.fullname.clone(),
            base_url: Some(effective_base.into()),
            ..Default::default()
        };
        config.save()?;

        if json {
            output::print_json_success(&serde_json::json!({
                "userid": info.userid,
                "fullname": info.fullname,
                "username": info.username,
            }));
        } else {
            println!(
                "{} {} ({})",
                "✓ 已登入:".green().bold(),
                info.fullname.unwrap_or_default(),
                info.username.unwrap_or_default()
            );
        }
        return Ok(());
    }

    if let Some(cookie) = session {
        // Session-based login
        let sp = if !json {
            Some(output::spinner("驗證 session..."))
        } else {
            None
        };

        let http_client = MoodleClient::new(
            Some(effective_base),
            AuthMethod::Session {
                cookie: cookie.clone(),
                sesskey: String::new(),
            },
        )?;

        let (sesskey, userid, fullname) = http_client.extract_sesskey(&cookie).await?;

        if let Some(sp) = sp {
            sp.finish_and_clear();
        }

        let config = E3Config {
            session: Some(cookie),
            sesskey: Some(sesskey),
            auth_mode: Some("session".into()),
            userid,
            fullname: fullname.clone(),
            base_url: Some(effective_base.into()),
            ..Default::default()
        };
        config.save()?;

        if json {
            output::print_json_success(&serde_json::json!({
                "userid": userid,
                "fullname": fullname,
            }));
        } else {
            println!(
                "{} {} (session)",
                "✓ 已登入:".green().bold(),
                fullname.unwrap_or_default()
            );
        }
        return Ok(());
    }

    // Username + password login
    let username = match username {
        Some(u) => u,
        None => {
            eprint!("Username: ");
            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .map_err(|e| E3Error::Other(format!("Input error: {e}")))?;
            input.trim().to_string()
        }
    };

    let password = match password {
        Some(p) => p,
        None => rpassword::prompt_password("Password: ")
            .map_err(|e| E3Error::Other(format!("Password input error: {e}")))?,
    };

    let sp = if !json {
        Some(output::spinner("登入中..."))
    } else {
        None
    };

    let token =
        MoodleClient::login_with_password(Some(effective_base), &username, &password).await?;

    let client = MoodleClient::new(Some(effective_base), AuthMethod::Token(token.clone()))?;
    let info = e3_core::auth::get_site_info(&client).await?;

    if let Some(sp) = sp {
        sp.finish_and_clear();
    }

    let config = E3Config {
        token: Some(token),
        auth_mode: Some("token".into()),
        userid: info.userid,
        fullname: info.fullname.clone(),
        base_url: Some(effective_base.into()),
        ..Default::default()
    };
    config.save()?;

    // Save credentials for auto-relogin
    save_env_credentials(&username, &password)?;

    if json {
        output::print_json_success(&serde_json::json!({
            "userid": info.userid,
            "fullname": info.fullname,
            "username": info.username,
        }));
    } else {
        println!(
            "{} {} ({})",
            "✓ 已登入:".green().bold(),
            info.fullname.unwrap_or_default(),
            info.username.unwrap_or_default()
        );
    }

    Ok(())
}

pub fn logout(json: bool) -> Result<()> {
    E3Config::delete()?;

    if json {
        output::print_json_success(&serde_json::json!({ "message": "已登出" }));
    } else {
        println!("{}", "✓ 已登出".green().bold());
    }

    Ok(())
}

pub async fn whoami(json: bool, base_url: Option<&str>) -> Result<()> {
    let config = E3Config::load()?;
    let client = config.build_client(base_url)?;

    let sp = if !json {
        Some(output::spinner("查詢中..."))
    } else {
        None
    };
    let info = e3_core::auth::get_site_info(&client).await?;
    if let Some(sp) = sp {
        sp.finish_and_clear();
    }

    if json {
        output::print_json_success(&info);
    } else {
        println!("{}: {}", "使用者".bold(), info.fullname.unwrap_or_default());
        println!("{}: {}", "帳號".bold(), info.username.unwrap_or_default());
        println!("{}: {}", "ID".bold(), info.userid.unwrap_or(0));
        println!("{}: {}", "網站".bold(), info.siteurl.unwrap_or_default());
        println!(
            "{}: {}",
            "認證方式".bold(),
            config.auth_mode.unwrap_or_default()
        );
    }

    Ok(())
}
