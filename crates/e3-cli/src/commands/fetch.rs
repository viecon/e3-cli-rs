use crate::config::build_client_with_relogin;
use crate::output;
use e3_core::error::Result;

pub async fn run(json: bool, base_url: Option<&str>, url: &str) -> Result<()> {
    let (client, _config) = build_client_with_relogin(base_url).await?;

    let sp = if !json {
        Some(output::spinner("取得頁面..."))
    } else {
        None
    };

    let html = client.fetch_page(url).await?;

    if let Some(sp) = sp {
        sp.finish_and_clear();
    }

    // Extract #region-main content if present
    let content = extract_region_main(&html).unwrap_or(&html);
    let text = output::strip_html(content);

    if json {
        output::print_json_success(&serde_json::json!({
            "url": url,
            "content": text,
            "html": content,
        }));
    } else {
        println!("{}", text);
    }

    Ok(())
}

/// Extract content within <div ... id="region-main" ...>...</div>
fn extract_region_main(html: &str) -> Option<&str> {
    // Find the opening tag
    let pattern = r#"id="region-main""#;
    let tag_attr_pos = html.find(pattern)?;

    // Walk back to find the opening <div or <section
    let before = &html[..tag_attr_pos];
    let open_tag_start = before.rfind('<')?;

    // Find the end of the opening tag
    let after_attr = &html[tag_attr_pos..];
    let tag_end = after_attr.find('>')?;
    let content_start = tag_attr_pos + tag_end + 1;

    // Find matching closing tag using depth counting
    let tag_name = &html[open_tag_start + 1..open_tag_start + 1 + 3]; // "div" or "sec"
    let close_tag = if tag_name.starts_with("div") {
        "</div>"
    } else if tag_name.starts_with("sec") {
        "</section>"
    } else {
        "</div>"
    };

    let open_tag = if close_tag == "</div>" {
        "<div"
    } else {
        "<section"
    };

    let region = &html[content_start..];
    let mut depth = 1i32;
    let mut pos = 0;

    while depth > 0 && pos < region.len() {
        let next_open = region[pos..].find(open_tag).map(|i| i + pos);
        let next_close = region[pos..].find(close_tag).map(|i| i + pos);

        match (next_open, next_close) {
            (Some(o), Some(c)) if o < c => {
                depth += 1;
                pos = o + open_tag.len();
            }
            (_, Some(c)) => {
                depth -= 1;
                if depth == 0 {
                    return Some(&html[content_start..content_start + c]);
                }
                pos = c + close_tag.len();
            }
            _ => break,
        }
    }

    None
}
