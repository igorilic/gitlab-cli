use anyhow::Result;
use serde_json::json;
use tracing::debug;

use super::client::GitLabClient;
use crate::models::project::Project;

pub struct ProjectsApi<'a> {
    client: &'a GitLabClient,
}

impl<'a> ProjectsApi<'a> {
    pub fn new(client: &'a GitLabClient) -> Self {
        Self { client }
    }

    pub async fn get_by_id(&self, id: u64) -> Result<Project> {
        let url = format!("{}/projects/{}", self.client.api_url(), id);

        debug!("Fetching project by ID: {}", id);

        let response = self
            .client
            .http_client()
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<Project>()
            .await?;

        Ok(response)
    }

    pub async fn get_by_path(&self, path: &str) -> Result<Project> {
        // URL encode the path
        let encoded_path = urlencoding::encode(path);
        let url = format!("{}/projects/{}", self.client.api_url(), encoded_path);

        debug!("Fetching project by path: {}", path);

        let response = self
            .client
            .http_client()
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<Project>()
            .await?;

        Ok(response)
    }

    pub async fn find_by_topic(&self, topic: &str) -> Result<Vec<Project>> {
        let url = format!(
            "{}/projects?topic={}&per_page=100",
            self.client.api_url(),
            urlencoding::encode(topic)
        );

        debug!("Searching for projects with topic: {}", topic);

        let mut all_projects = Vec::new();
        let mut page = 1;

        loop {
            let page_url = format!("{}&page={}", url, page);

            let projects: Vec<Project> = self
                .client
                .http_client()
                .get(&page_url)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;

            if projects.is_empty() {
                break;
            }

            let count = projects.len();
            all_projects.extend(projects);

            debug!("Retrieved {} projects on page {}", count, page);

            page += 1;
        }

        debug!(
            "Found a total of {} projects with topic '{}'",
            all_projects.len(),
            topic
        );

        Ok(all_projects)
    }

    pub async fn update_topics(&self, project_id: u64, topics: &[String]) -> Result<Project> {
        let url = format!("{}/projects/{}", self.client.api_url(), project_id);

        debug!("Updating topics for project {}: {:?}", project_id, topics);

        let body = json!({
            "topics": topics,
        });

        let response = self
            .client
            .http_client()
            .put(&url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json::<Project>()
            .await?;

        Ok(response)
    }
    // In src/gitlab/projects.rs:
    pub async fn list(&self) -> Result<Vec<Project>> {
        let url = format!("{}/projects?per_page=100", self.client.api_url());

        debug!("Listing all projects");

        let mut all_projects = Vec::new();
        let mut page = 1;

        loop {
            let page_url = format!("{}&page={}", url, page);

            let projects: Vec<Project> = self
                .client
                .http_client()
                .get(&page_url)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;

            if projects.is_empty() {
                break;
            }

            let count = projects.len();
            all_projects.extend(projects);

            debug!("Retrieved {} projects on page {}", count, page);

            page += 1;
        }

        debug!("Found a total of {} projects", all_projects.len());

        Ok(all_projects)
    }
}
