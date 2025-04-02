use anyhow::{Context, Result};
use csv::Reader;
use std::fs::File;
use std::path::Path;
use tracing::debug;

use crate::models::{project::Project, user::User};

#[derive(Debug)]
pub struct CsvReader {
    path: std::path::PathBuf,
}

impl CsvReader {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        debug!("Creating CSV reader for file: {:?}", path);
        // Check if the file exists before returning the reader
        if !path.exists() {
            anyhow::bail!("CSV file not found: {:?}", path);
        }
        Ok(Self { path })
    }

    pub fn read_users(&self) -> Result<Vec<User>> {
        debug!("Reading users from CSV file: {:?}", self.path);

        let file = File::open(&self.path)
            .with_context(|| format!("Failed to open CSV file: {:?}", self.path))?;

        let mut reader = Reader::from_reader(file);
        let mut users = Vec::new();

        for result in reader.deserialize() {
            let record: UserRecord =
                result.with_context(|| "Failed to parse user record from CSV")?;

            // Convert to User model
            let user = User {
                id: record.id,
                username: record.username,
                name: record.name,
                state: "active".to_string(), // Assume active by default
                email: record.email,
            };

            users.push(user);
        }

        debug!("Read {} users from CSV file", users.len());
        Ok(users)
    }

    pub fn read_projects(&self) -> Result<Vec<Project>> {
        debug!("Reading projects from CSV file: {:?}", self.path);

        let file = File::open(&self.path)
            .with_context(|| format!("Failed to open CSV file: {:?}", self.path))?;

        let mut reader = Reader::from_reader(file);
        let mut projects = Vec::new();

        for result in reader.deserialize() {
            let record: ProjectRecord =
                result.with_context(|| "Failed to parse project record from CSV")?;

            // Parse topics as comma-separated list
            let topics = if let Some(topics_str) = &record.topics {
                topics_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            } else {
                Vec::new()
            };

            // Convert to Project model
            let project = Project {
                id: record.id,
                path_with_namespace: record.path_with_namespace,
                name: record.name,
                description: record.description,
                default_branch: record.default_branch,
                visibility: record.visibility.unwrap_or_else(|| "private".to_string()),
                web_url: record.web_url.unwrap_or_default(),
                topics,
            };

            projects.push(project);
        }

        debug!("Read {} projects from CSV file", projects.len());
        Ok(projects)
    }
}

#[derive(Debug, serde::Deserialize)]
struct UserRecord {
    id: u64,
    username: String,
    name: String,
    #[serde(default)]
    email: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ProjectRecord {
    id: u64,
    path_with_namespace: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    default_branch: Option<String>,
    #[serde(default)]
    visibility: Option<String>,
    #[serde(default)]
    web_url: Option<String>,
    #[serde(default)]
    topics: Option<String>,
}
