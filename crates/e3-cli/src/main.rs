#![allow(dead_code)]

mod commands;
mod config;
mod output;

use clap::Parser;
use commands::Commands;

#[derive(Parser)]
#[command(name = "e3", version, about = "NYCU E3 CLI — Moodle 助手工具")]
pub struct Cli {
    /// Output as JSON
    #[arg(long, global = true)]
    json: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    /// Override base URL
    #[arg(long, global = true)]
    base_url: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if cli.no_color || std::env::var("NO_COLOR").is_ok() {
        colored::control::set_override(false);
    }

    let result = commands::run(cli).await;

    if let Err(e) = result {
        if std::env::args().any(|a| a == "--json") {
            output::print_json_error(&e);
        } else {
            output::print_error(&e);
        }
        std::process::exit(1);
    }
}
