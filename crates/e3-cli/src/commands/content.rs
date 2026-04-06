use crate::config::build_client_with_relogin;
use crate::output;
use colored::Colorize;
use e3_core::error::Result;

pub async fn run(json: bool, base_url: Option<&str>, course_id: i64) -> Result<()> {
    let (client, _config) = build_client_with_relogin(base_url).await?;

    let sp = if !json {
        Some(output::spinner("取得課程內容..."))
    } else {
        None
    };

    let sections = e3_core::courses::get_course_contents(&client, course_id).await?;

    if let Some(sp) = sp {
        sp.finish_and_clear();
    }

    if json {
        output::print_json_success(&sections);
        return Ok(());
    }

    if sections.is_empty() {
        println!("{}", "課程沒有內容".dimmed());
        return Ok(());
    }

    for section in &sections {
        let section_name = section.name.as_deref().unwrap_or("(未命名)");
        println!("{}", section_name.bold().cyan());

        for module in &section.modules {
            let name = module.name.as_deref().unwrap_or("?");
            let modname = module.modname.as_deref().unwrap_or("?");
            let icon = module_icon(modname);

            let visible = module.visible.unwrap_or(1) != 0;
            if !visible {
                continue;
            }

            print!("  {} {}", icon, name);

            // Show due date for assignments
            if let Some(desc) = &module.description {
                let plain = output::strip_html(desc);
                if !plain.trim().is_empty() {
                    let preview: String = plain.chars().take(80).collect();
                    let preview = preview.replace('\n', " ");
                    print!(" — {}", preview.dimmed());
                }
            }

            println!();

            // Show attached files
            for file in &module.contents {
                if let Some(filename) = &file.filename {
                    let size = file
                        .filesize
                        .map(|s| format!(" ({})", output::format_size(s)))
                        .unwrap_or_default();
                    println!("    {} {}{}", "•".dimmed(), filename, size.dimmed());
                }
            }
        }
        println!();
    }

    Ok(())
}

fn module_icon(modname: &str) -> &str {
    match modname {
        "assign" => "\u{1f4dd}",    // memo
        "forum" => "\u{1f4ac}",     // speech bubble
        "resource" => "\u{1f4c4}",  // page
        "folder" => "\u{1f4c1}",    // folder
        "url" => "\u{1f517}",       // link
        "page" => "\u{1f4c3}",      // page with curl
        "quiz" => "\u{2753}",       // question mark
        "label" => "\u{1f3f7}",     // label
        "feedback" => "\u{1f4cb}",  // clipboard
        "choice" => "\u{2611}",     // ballot box
        "glossary" => "\u{1f4d6}",  // book
        "workshop" => "\u{1f6e0}",  // hammer and wrench
        "wiki" => "\u{1f4da}",      // books
        _ => "\u{25aa}",            // small black square
    }
}

