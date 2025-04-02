use anyhow::Result;
use clap::{Args, Subcommand};
use colored::Colorize;
use tracing::{debug, info};

use crate::gitlab::client::GitLabClient;

#[derive(Args)]
pub struct ProjectsCommands {
    #[command(subcommand)]
    command: ProjectsSubcommands,
}

#[derive(Subcommand)]
enum ProjectsSubcommands {
    /// List GitLab projects
    List(ListProjectsArgs),
}

#[derive(Args)]
struct ListProjectsArgs {
    /// Filter projects by topic
    #[arg(short, long)]
    topic: Option<String>,

    /// Output format (simple, detailed)
    #[arg(short, long, default_value = "simple")]
    format: String,
}

impl ProjectsCommands {
    pub async fn execute(&self, client: &GitLabClient) -> Result<()> {
        match &self.command {
            ProjectsSubcommands::List(args) => self.list_projects(client, args).await,
        }
    }

    async fn list_projects(&self, client: &GitLabClient, args: &ListProjectsArgs) -> Result<()> {
        info!("Listing GitLab projects");

        let projects = if let Some(topic) = &args.topic {
            debug!("Filtering projects by topic: {}", topic);
            client.projects().find_by_topic(topic).await?
        } else {
            debug!("Retrieving all projects");
            client.projects().list().await?
        };

        if projects.is_empty() {
            println!("No projects found.");
            return Ok(());
        }

        println!("Found {} projects:", projects.len());

        let is_detailed = args.format.to_lowercase() == "detailed";

        for project in &projects {
            if is_detailed {
                println!("---------------------------");
                println!("ID: {}", project.id);
                println!("Name: {}", project.name);
                println!("Path: {}", project.path_with_namespace);

                if let Some(desc) = &project.description {
                    println!("Description: {}", desc);
                }

                println!("Visibility: {}", project.visibility);
                println!("Web URL: {}", project.web_url);

                if !project.topics.is_empty() {
                    println!("Topics:");
                    for topic in &project.topics {
                        println!("  - {}", topic);
                    }
                } else {
                    println!("Topics: None");
                }
                println!("---------------------------");
            } else {
                // Simple format
                let topics_str = if !project.topics.is_empty() {
                    project.topics.join(", ")
                } else {
                    "No topics".to_string()
                };

                println!(
                    "{} - {} [{}]",
                    project.id.to_string().cyan(),
                    project.path_with_namespace.green(),
                    topics_str.yellow()
                );
            }
        }

        Ok(())
    }
}
