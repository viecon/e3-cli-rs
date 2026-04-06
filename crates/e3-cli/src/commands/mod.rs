pub mod assignments;
pub mod auth;
pub mod calendar;
pub mod config;
pub mod courses;
pub mod download;
pub mod export;
pub mod grades;
pub mod news;
pub mod notifications;
pub mod open;
pub mod status;
pub mod updates;
pub mod upload;

use crate::Cli;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Login to E3
    Login {
        /// Username (student ID)
        #[arg(short, long)]
        username: Option<String>,
        /// Password
        #[arg(short, long)]
        password: Option<String>,
        /// Use web service token directly
        #[arg(long)]
        token: Option<String>,
        /// Use MoodleSession cookie
        #[arg(long)]
        session: Option<String>,
    },
    /// Logout — clear saved credentials
    Logout,
    /// Show current user info
    Whoami,
    /// Dashboard: pending assignments + notifications + courses
    Status,
    /// List enrolled courses
    Courses {
        /// Show all courses (including past)
        #[arg(long)]
        all: bool,
    },
    /// List pending assignments
    Assignments {
        /// Days ahead to look (default: 30)
        #[arg(long, default_value = "30")]
        days: i64,
    },
    /// View submission details
    Submission {
        /// Assignment ID (cmid)
        id: i64,
    },
    /// View grades
    Grades {
        /// Course ID (omit for all courses overview)
        course: Option<i64>,
    },
    /// View calendar events
    Calendar {
        /// Days ahead (default: 30)
        #[arg(long, default_value = "30")]
        days: i64,
        /// Generate ICS file
        #[arg(long)]
        ics: Option<Option<String>>,
        /// Days for ICS generation (default: 90)
        #[arg(long, default_value = "90")]
        ics_days: i64,
    },
    /// View course announcements
    News {
        /// Filter by course ID
        #[arg(long)]
        course: Option<i64>,
        /// Days back (default: 7)
        #[arg(long, default_value = "7")]
        days: i64,
    },
    /// View notifications
    Notifications {
        /// Max number of notifications (default: 20)
        #[arg(long, default_value = "20")]
        limit: i32,
    },
    /// View recent course updates
    Updates {
        /// Course ID (omit for all courses)
        course: Option<i64>,
        /// Days back (default: 7)
        #[arg(long, default_value = "7")]
        days: i64,
    },
    /// Download course files
    Download {
        /// Course ID (or --all)
        course: Option<i64>,
        /// Download all courses
        #[arg(long)]
        all: bool,
        /// Filter by file extensions (comma-separated)
        #[arg(long, value_delimiter = ',')]
        r#type: Option<Vec<String>>,
        /// Output directory
        #[arg(short, long)]
        output: Option<String>,
        /// List files only (don't download)
        #[arg(long)]
        list: bool,
        /// Skip existing files
        #[arg(long)]
        skip_existing: bool,
    },
    /// Upload files and submit assignment
    Upload {
        /// Assignment ID (cmid)
        id: i64,
        /// Files to upload
        files: Vec<String>,
        /// Upload only, don't submit
        #[arg(long)]
        no_submit: bool,
    },
    /// Export data to CSV
    Export {
        /// What to export: grades, assignments
        target: String,
        /// Output file
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Manage config
    Config {
        /// Action: get, set, list
        action: Option<String>,
        /// Config key
        key: Option<String>,
        /// Config value
        value: Option<String>,
    },
    /// Open E3 page in browser
    Open {
        /// Target: dashboard, calendar, grades, or course name/id
        target: Option<String>,
    },
    /// Generate shell completions
    Completions {
        /// Shell: bash, zsh, fish, powershell
        shell: String,
    },
}

pub async fn run(cli: Cli) -> e3_core::error::Result<()> {
    match cli.command {
        Commands::Login {
            username,
            password,
            token,
            session,
        } => {
            auth::login(
                cli.json,
                cli.base_url.as_deref(),
                username,
                password,
                token,
                session,
            )
            .await
        }
        Commands::Logout => auth::logout(cli.json),
        Commands::Whoami => auth::whoami(cli.json, cli.base_url.as_deref()).await,
        Commands::Status => status::run(cli.json, cli.base_url.as_deref()).await,
        Commands::Courses { all } => courses::run(cli.json, cli.base_url.as_deref(), all).await,
        Commands::Assignments { days } => {
            assignments::run(cli.json, cli.base_url.as_deref(), days).await
        }
        Commands::Submission { id } => {
            assignments::submission(cli.json, cli.base_url.as_deref(), id).await
        }
        Commands::Grades { course } => grades::run(cli.json, cli.base_url.as_deref(), course).await,
        Commands::Calendar {
            days,
            ics,
            ics_days,
        } => calendar::run(cli.json, cli.base_url.as_deref(), days, ics, ics_days).await,
        Commands::News { course, days } => {
            news::run(cli.json, cli.base_url.as_deref(), course, days).await
        }
        Commands::Notifications { limit } => {
            notifications::run(cli.json, cli.base_url.as_deref(), limit).await
        }
        Commands::Updates { course, days } => {
            updates::run(cli.json, cli.base_url.as_deref(), course, days).await
        }
        Commands::Download {
            course,
            all,
            r#type,
            output,
            list,
            skip_existing,
        } => {
            download::run(
                cli.json,
                cli.base_url.as_deref(),
                course,
                all,
                r#type,
                output,
                list,
                skip_existing,
            )
            .await
        }
        Commands::Upload {
            id,
            files,
            no_submit,
        } => upload::run(cli.json, cli.base_url.as_deref(), id, files, no_submit).await,
        Commands::Export { target, output } => {
            export::run(cli.json, cli.base_url.as_deref(), &target, output).await
        }
        Commands::Config { action, key, value } => config::run(cli.json, action, key, value),
        Commands::Open { target } => open::run(cli.base_url.as_deref(), target),
        Commands::Completions { shell } => {
            use clap::CommandFactory;
            let shell: clap_complete::Shell = shell
                .parse()
                .map_err(|_| e3_core::E3Error::Other("Invalid shell".into()))?;
            clap_complete::generate(shell, &mut Cli::command(), "e3", &mut std::io::stdout());
            Ok(())
        }
    }
}
