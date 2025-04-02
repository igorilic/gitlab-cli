use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

mod commands;
mod gitlab;
mod models;
mod utils;

use commands::{
    file::FileCommands, projects::ProjectsCommands, topics::TopicsCommands, user::UserCommands,
};

#[derive(Parser)]
#[command(
    name = "gitlab-cli",
    about = "Bulk management tool for GitLab repositories",
    version,
    author
)]
struct Cli {
    /// GitLab API URL (e.g., https://gitlab.example.com/api/v4)
    #[arg(long, env = "GITLAB_API_URL")]
    api_url: Option<String>,

    /// GitLab API token with appropriate permissions
    #[arg(long, env = "GITLAB_API_TOKEN")]
    api_token: Option<String>,

    /// Verbose output mode
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage users in GitLab projects
    User(UserCommands),

    /// Alias for user management
    Users(UserCommands),

    /// Manage files in GitLab repositories
    File(FileCommands),

    /// Manage topics in GitLab projects
    Topics(TopicsCommands),

    /// Manage projects in GitLab
    Projects(ProjectsCommands),
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    info!("Starting GitLab bulk management CLI");

    // Get API URL and token from env vars if not provided
    let api_url = cli.api_url.unwrap_or_else(|| {
        std::env::var("GITLAB_API_URL")
            .expect("GITLAB_API_URL must be provided via argument or environment variable")
    });

    let api_token = cli.api_token.unwrap_or_else(|| {
        std::env::var("GITLAB_API_TOKEN")
            .expect("GITLAB_API_TOKEN must be provided via argument or environment variable")
    });

    // Create GitLab client
    let client = gitlab::client::GitLabClient::new(&api_url, &api_token);

    // Execute the selected command
    match cli.command {
        Commands::User(cmd) => cmd.execute(&client).await?,
        Commands::Users(cmd) => cmd.execute(&client).await?,
        Commands::File(cmd) => cmd.execute(&client).await?,
        Commands::Topics(cmd) => cmd.execute(&client).await?,
        Commands::Projects(cmd) => cmd.execute(&client).await?,
    }
    info!("GitLab bulk management CLI completed successfully");

    Ok(())
}
