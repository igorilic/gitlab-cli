use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::path::PathBuf;
use tracing::{debug, info};

use crate::gitlab::client::GitLabClient;
use crate::utils::csv::CsvReader;

#[derive(Args)]
pub struct TopicsCommands {
    #[command(subcommand)]
    command: TopicsSubcommands,
}

#[derive(Subcommand)]
enum TopicsSubcommands {
    /// Add topics to projects
    Add(AddTopicsArgs),

    /// Remove topics from projects
    Remove(RemoveTopicsArgs),

    /// List topics for projects
    List(ListTopicsArgs),
}

#[derive(Args)]
struct AddTopicsArgs {
    /// Topics to add in format "key:value" (e.g., "team:backend")
    #[arg(required = true, value_delimiter = ',')]
    topics: Vec<String>,

    /// Path to CSV file containing project details
    #[arg(short, long, conflicts_with = "project_ids")]
    project_file: Option<PathBuf>,

    /// Comma-separated list of project IDs or paths
    #[arg(short, long, value_delimiter = ',', conflicts_with = "project_file")]
    project_ids: Option<Vec<String>>,

    /// Existing GitLab topic to filter projects
    #[arg(short, long)]
    filter_topic: Option<String>,
}

#[derive(Args)]
struct RemoveTopicsArgs {
    /// Topics to remove in format "key:value" (e.g., "team:backend")
    #[arg(required = true, value_delimiter = ',')]
    topics: Vec<String>,

    /// Path to CSV file containing project details
    #[arg(short, long, conflicts_with = "project_ids")]
    project_file: Option<PathBuf>,

    /// Comma-separated list of project IDs or paths
    #[arg(short, long, value_delimiter = ',', conflicts_with = "project_file")]
    project_ids: Option<Vec<String>>,

    /// Existing GitLab topic to filter projects
    #[arg(short, long)]
    filter_topic: Option<String>,
}

#[derive(Args)]
struct ListTopicsArgs {
    /// Path to CSV file containing project details
    #[arg(short, long, conflicts_with = "project_ids")]
    project_file: Option<PathBuf>,

    /// Comma-separated list of project IDs or paths
    #[arg(short, long, value_delimiter = ',', conflicts_with = "project_file")]
    project_ids: Option<Vec<String>>,

    /// Existing GitLab topic to filter projects
    #[arg(short, long)]
    filter_topic: Option<String>,
}

impl TopicsCommands {
    pub async fn execute(&self, client: &GitLabClient) -> Result<()> {
        match &self.command {
            TopicsSubcommands::Add(args) => self.add_topics(client, args).await,
            TopicsSubcommands::Remove(args) => self.remove_topics(client, args).await,
            TopicsSubcommands::List(args) => self.list_topics(client, args).await,
        }
    }

    async fn add_topics(&self, client: &GitLabClient, args: &AddTopicsArgs) -> Result<()> {
        info!("Adding topics to projects");

        // Get projects from file, command line, or by topic
        let projects = if let Some(file_path) = &args.project_file {
            debug!("Loading projects from file: {:?}", file_path);
            self.load_projects_from_file(file_path)?
        } else if let Some(project_ids) = &args.project_ids {
            debug!("Using project IDs from command line: {:?}", project_ids);
            self.resolve_project_ids(client, project_ids).await?
        } else if let Some(topic) = &args.filter_topic {
            debug!("Searching for projects with topic: {}", topic);
            client.projects().find_by_topic(topic).await?
        } else {
            anyhow::bail!(
                "Either --project-file, --project-ids, or --filter-topic must be provided"
            );
        };

        info!("Found {} projects to modify", projects.len());

        // Parse and validate topics
        let topics: Vec<String> = args
            .topics
            .iter()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();

        if topics.is_empty() {
            anyhow::bail!("No valid topics provided");
        }

        // Add topics to projects
        for project in &projects {
            info!(
                "Adding topics to project {}: {:?}",
                project.path_with_namespace, topics
            );

            // Get current topics for the project
            let mut current_topics = project.topics.clone();

            // Add new topics
            for topic in &topics {
                if !current_topics.contains(topic) {
                    current_topics.push(topic.clone());
                }
            }

            // Update project topics
            client
                .projects()
                .update_topics(project.id, &current_topics)
                .await
                .with_context(|| {
                    format!(
                        "Failed to update topics for project {}",
                        project.path_with_namespace
                    )
                })?;
        }

        info!("Successfully added topics to projects");
        Ok(())
    }

    async fn remove_topics(&self, client: &GitLabClient, args: &RemoveTopicsArgs) -> Result<()> {
        info!("Removing topics from projects");

        // Get projects from file, command line, or by topic
        let projects = if let Some(file_path) = &args.project_file {
            debug!("Loading projects from file: {:?}", file_path);
            self.load_projects_from_file(file_path)?
        } else if let Some(project_ids) = &args.project_ids {
            debug!("Using project IDs from command line: {:?}", project_ids);
            self.resolve_project_ids(client, project_ids).await?
        } else if let Some(topic) = &args.filter_topic {
            debug!("Searching for projects with topic: {}", topic);
            client.projects().find_by_topic(topic).await?
        } else {
            anyhow::bail!(
                "Either --project-file, --project-ids, or --filter-topic must be provided"
            );
        };

        info!("Found {} projects to modify", projects.len());

        // Parse and validate topics
        let topics_to_remove: Vec<String> = args
            .topics
            .iter()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();

        if topics_to_remove.is_empty() {
            anyhow::bail!("No valid topics provided");
        }

        // Remove topics from projects
        for project in &projects {
            info!(
                "Removing topics from project {}: {:?}",
                project.path_with_namespace, topics_to_remove
            );

            // Get current topics for the project
            let current_topics = project.topics.clone();

            // Filter out topics to remove
            let updated_topics: Vec<String> = current_topics
                .into_iter()
                .filter(|t| !topics_to_remove.contains(t))
                .collect();

            // Update project topics
            client
                .projects()
                .update_topics(project.id, &updated_topics)
                .await
                .with_context(|| {
                    format!(
                        "Failed to update topics for project {}",
                        project.path_with_namespace
                    )
                })?;
        }

        info!("Successfully removed topics from projects");
        Ok(())
    }

    async fn list_topics(&self, client: &GitLabClient, args: &ListTopicsArgs) -> Result<()> {
        info!("Listing topics for projects");

        // Get projects from file, command line, or by topic
        let projects = if let Some(file_path) = &args.project_file {
            debug!("Loading projects from file: {:?}", file_path);
            self.load_projects_from_file(file_path)?
        } else if let Some(project_ids) = &args.project_ids {
            debug!("Using project IDs from command line: {:?}", project_ids);
            self.resolve_project_ids(client, project_ids).await?
        } else if let Some(topic) = &args.filter_topic {
            debug!("Searching for projects with topic: {}", topic);
            client.projects().find_by_topic(topic).await?
        } else {
            anyhow::bail!(
                "Either --project-file, --project-ids, or --filter-topic must be provided"
            );
        };

        // Print topics for each project
        println!("Topics for {} projects:", projects.len());
        println!("---------------------------");

        for project in &projects {
            println!(
                "Project: {} (ID: {})",
                project.path_with_namespace, project.id
            );

            if project.topics.is_empty() {
                println!("  No topics assigned");
            } else {
                for topic in &project.topics {
                    println!("  - {}", topic);
                }
            }

            println!("---------------------------");
        }

        Ok(())
    }

    fn load_projects_from_file(
        &self,
        file_path: &PathBuf,
    ) -> Result<Vec<crate::models::project::Project>> {
        let extension = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

        if extension.to_lowercase() == "csv" {
            let reader = CsvReader::new(file_path)?;
            reader.read_projects()
        } else {
            anyhow::bail!(
                "Unsupported file format: {}. Only CSV files are supported.",
                extension
            )
        }
    }

    async fn resolve_project_ids(
        &self,
        client: &GitLabClient,
        project_ids: &[String],
    ) -> Result<Vec<crate::models::project::Project>> {
        let mut projects = Vec::new();

        for id_or_path in project_ids {
            // Try to parse as ID first
            if let Ok(id) = id_or_path.parse::<u64>() {
                let project = client.projects().get_by_id(id).await?;
                projects.push(project);
            } else {
                // If not an ID, treat as path
                let project = client.projects().get_by_path(id_or_path).await?;
                projects.push(project);
            }
        }

        Ok(projects)
    }
}
