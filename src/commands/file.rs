use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::path::PathBuf;
use tracing::{debug, info};

use crate::gitlab::client::GitLabClient;
use crate::utils::csv::CsvReader;

#[derive(Args)]
pub struct FileCommands {
    #[command(subcommand)]
    command: FileSubcommands,
}

#[derive(Subcommand)]
enum FileSubcommands {
    /// Add or update files in repositories
    Update(UpdateFileArgs),
}

#[derive(Args)]
struct UpdateFileArgs {
    /// Path to local file to upload
    #[arg(short, long)]
    file_path: PathBuf,

    /// Target path in the repository
    #[arg(short, long)]
    target_path: String,

    /// Commit message
    #[arg(short, long, default_value = "Update file via gitlab-bulk CLI")]
    commit_message: String,

    /// Branch name (defaults to the default branch of the repository)
    #[arg(short, long)]
    branch: Option<String>,

    /// Path to CSV file containing project details
    #[arg(long, conflicts_with = "project_ids")]
    project_file: Option<PathBuf>,

    /// Comma-separated list of project IDs or paths
    #[arg(short, long, value_delimiter = ',', conflicts_with = "project_file")]
    project_ids: Option<Vec<String>>,

    /// GitLab topic to filter projects
    #[arg(short, long)]
    topic: Option<String>,

    /// Content changes to apply (format: "old_string:new_string")
    #[arg(short, long, value_delimiter = ';')]
    changes: Option<Vec<String>>,
}

impl FileCommands {
    pub async fn execute(&self, client: &GitLabClient) -> Result<()> {
        match &self.command {
            FileSubcommands::Update(args) => self.update_files(client, args).await,
        }
    }

    async fn update_files(&self, client: &GitLabClient, args: &UpdateFileArgs) -> Result<()> {
        info!("Updating files in repositories");

        // Get projects from file, command line, or by topic
        let projects = if let Some(file_path) = &args.project_file {
            debug!("Loading projects from file: {:?}", file_path);
            self.load_projects_from_file(file_path)?
        } else if let Some(project_ids) = &args.project_ids {
            debug!("Using project IDs from command line: {:?}", project_ids);
            self.resolve_project_ids(client, project_ids).await?
        } else if let Some(topic) = &args.topic {
            debug!("Searching for projects with topic: {}", topic);
            client.projects().find_by_topic(topic).await?
        } else {
            anyhow::bail!("Either --project-file, --project-ids, or --topic must be provided");
        };

        info!("Found {} projects to update", projects.len());

        // Read file content
        let mut content = std::fs::read_to_string(&args.file_path)
            .with_context(|| format!("Failed to read file: {:?}", args.file_path))?;

        // Apply content changes if provided
        if let Some(changes) = &args.changes {
            for change in changes {
                let parts: Vec<&str> = change.split(':').collect();
                if parts.len() == 2 {
                    let old_str = parts[0];
                    let new_str = parts[1];
                    content = content.replace(old_str, new_str);
                } else {
                    debug!("Ignoring invalid change format: {}", change);
                }
            }
        }

        // Update file in each project
        for project in &projects {
            info!("Updating file in project: {}", project.path_with_namespace);

            // Get the default branch if none specified
            let branch = if let Some(branch) = &args.branch {
                branch.clone()
            } else {
                project
                    .default_branch
                    .clone()
                    .unwrap_or_else(|| "main".to_string())
            };

            // Check if file exists first
            let file_exists = client
                .files()
                .file_exists(project.id, &args.target_path, &branch)
                .await?;

            if file_exists {
                debug!("File exists, updating: {}", args.target_path);
                client
                    .files()
                    .update_file(
                        project.id,
                        &args.target_path,
                        &branch,
                        &args.commit_message,
                        &content,
                    )
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to update file {} in project {}",
                            args.target_path, project.path_with_namespace
                        )
                    })?;
            } else {
                debug!("File doesn't exist, creating: {}", args.target_path);
                client
                    .files()
                    .create_file(
                        project.id,
                        &args.target_path,
                        &branch,
                        &args.commit_message,
                        &content,
                    )
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to create file {} in project {}",
                            args.target_path, project.path_with_namespace
                        )
                    })?;
            }
        }

        info!("Successfully updated files in repositories");
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

