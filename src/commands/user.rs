use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::path::PathBuf;
use tracing::{debug, info};

use crate::gitlab::client::GitLabClient;
use crate::models::user::AccessLevel;
use crate::utils::csv::CsvReader;

#[derive(Args)]
pub struct UserCommands {
    #[command(subcommand)]
    command: UserSubcommands,
}

#[derive(Subcommand)]
enum UserSubcommands {
    /// Add users to projects
    Add(AddUserArgs),

    /// Remove users from projects
    Remove(RemoveUserArgs),
}

#[derive(Args)]
struct AddUserArgs {
    /// Path to CSV file containing user details
    #[arg(short, long, conflicts_with = "user_ids")]
    user_file: Option<PathBuf>,

    /// Comma-separated list of user IDs or usernames
    #[arg(short, long, value_delimiter = ',', conflicts_with = "user_file")]
    user_ids: Option<Vec<String>>,

    /// Path to CSV file containing project details
    #[arg(short, long, conflicts_with = "project_ids")]
    project_file: Option<PathBuf>,

    /// Comma-separated list of project IDs or paths
    #[arg(short, long, value_delimiter = ',', conflicts_with = "project_file")]
    project_ids: Option<Vec<String>>,

    /// GitLab topic to filter projects
    #[arg(short, long)]
    topic: Option<String>,

    /// Role/access level to grant (no-access, minimal-access, guest, planner, reporter, developer, maintainer, owner)
    #[arg(short, long, default_value = "maintainer")]
    role: AccessLevel,
}

#[derive(Args)]
struct RemoveUserArgs {
    /// Path to CSV file containing user details
    #[arg(short, long, conflicts_with = "user_ids")]
    user_file: Option<PathBuf>,

    /// Comma-separated list of user IDs or usernames
    #[arg(short, long, value_delimiter = ',', conflicts_with = "user_file")]
    user_ids: Option<Vec<String>>,

    /// Path to CSV file containing project details
    #[arg(short, long, conflicts_with = "project_ids")]
    project_file: Option<PathBuf>,

    /// Comma-separated list of project IDs or paths
    #[arg(short, long, value_delimiter = ',', conflicts_with = "project_file")]
    project_ids: Option<Vec<String>>,

    /// GitLab topic to filter projects
    #[arg(short, long)]
    topic: Option<String>,
}

impl UserCommands {
    pub async fn execute(&self, client: &GitLabClient) -> Result<()> {
        match &self.command {
            UserSubcommands::Add(args) => self.add_users(client, args).await,
            UserSubcommands::Remove(args) => self.remove_users(client, args).await,
        }
    }

    async fn add_users(&self, client: &GitLabClient, args: &AddUserArgs) -> Result<()> {
        info!("Adding users to projects");

        // Get users from file or command line
        let users = if let Some(file_path) = &args.user_file {
            debug!("Loading users from file: {:?}", file_path);
            self.load_users_from_file(file_path)?
        } else if let Some(user_ids) = &args.user_ids {
            debug!("Using user IDs from command line: {:?}", user_ids);
            self.resolve_user_ids(client, user_ids).await?
        } else {
            anyhow::bail!("Either --user-file or --user-ids must be provided");
        };

        info!("Found {} users to add", users.len());

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

        info!("Found {} projects to modify", projects.len());

        // Add users to projects
        for user in users {
            for project in &projects {
                info!(
                    "Adding user {} to project {}",
                    user.username, project.path_with_namespace
                );
                client
                    .users()
                    .add_to_project(user.id, project.id, args.role.clone())
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to add user {} to project {}",
                            user.username, project.path_with_namespace
                        )
                    })?;
            }
        }

        info!("Successfully added users to projects");
        Ok(())
    }

    async fn remove_users(&self, client: &GitLabClient, args: &RemoveUserArgs) -> Result<()> {
        info!("Removing users from projects");

        // Get users from file or command line
        let users = if let Some(file_path) = &args.user_file {
            debug!("Loading users from file: {:?}", file_path);
            self.load_users_from_file(file_path)?
        } else if let Some(user_ids) = &args.user_ids {
            debug!("Using user IDs from command line: {:?}", user_ids);
            self.resolve_user_ids(client, user_ids).await?
        } else {
            anyhow::bail!("Either --user-file or --user-ids must be provided");
        };

        info!("Found {} users to remove", users.len());

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

        info!("Found {} projects to modify", projects.len());

        // Remove users from projects
        for user in users {
            for project in &projects {
                info!(
                    "Removing user {} from project {}",
                    user.username, project.path_with_namespace
                );
                client
                    .users()
                    .remove_from_project(user.id, project.id)
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to remove user {} from project {}",
                            user.username, project.path_with_namespace
                        )
                    })?;
            }
        }

        info!("Successfully removed users from projects");
        Ok(())
    }

    fn load_users_from_file(&self, file_path: &PathBuf) -> Result<Vec<crate::models::user::User>> {
        let extension = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

        if extension.to_lowercase() == "csv" {
            let reader = CsvReader::new(file_path)?;
            reader.read_users()
        } else {
            anyhow::bail!(
                "Unsupported file format: {}. Only CSV files are supported.",
                extension
            )
        }
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

    async fn resolve_user_ids(
        &self,
        client: &GitLabClient,
        user_ids: &[String],
    ) -> Result<Vec<crate::models::user::User>> {
        let mut users = Vec::new();

        for id_or_username in user_ids {
            // Try to parse as ID first
            if let Ok(id) = id_or_username.parse::<u64>() {
                let user = client.users().get_by_id(id).await?;
                users.push(user);
            } else {
                // If not an ID, treat as username
                let user = client.users().get_by_username(id_or_username).await?;
                users.push(user);
            }
        }

        Ok(users)
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

