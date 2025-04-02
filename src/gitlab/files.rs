use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::Deserialize;
use serde_json::json;
use tracing::debug;

use super::client::GitLabClient;

pub struct FilesApi<'a> {
    client: &'a GitLabClient,
}

#[derive(Deserialize)]
struct FileResponse {
    file_path: String,
    content: String, // Base64 encoded
}

impl<'a> FilesApi<'a> {
    pub fn new(client: &'a GitLabClient) -> Self {
        Self { client }
    }

    pub async fn file_exists(
        &self,
        project_id: u64,
        file_path: &str,
        branch: &str,
    ) -> Result<bool> {
        let encoded_path = urlencoding::encode(file_path);
        let url = format!(
            "{}/projects/{}/repository/files/{}?ref={}",
            self.client.api_url(),
            project_id,
            encoded_path,
            branch
        );

        debug!(
            "Checking if file exists: {} in project {} branch {}",
            file_path, project_id, branch
        );

        let response = self.client.http_client().get(&url).send().await;

        match response {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    pub async fn get_file_content(
        &self,
        project_id: u64,
        file_path: &str,
        branch: &str,
    ) -> Result<String> {
        let encoded_path = urlencoding::encode(file_path);
        let url = format!(
            "{}/projects/{}/repository/files/{}?ref={}",
            self.client.api_url(),
            project_id,
            encoded_path,
            branch
        );

        debug!(
            "Fetching file content: {} in project {} branch {}",
            file_path, project_id, branch
        );

        let response: FileResponse = self
            .client
            .http_client()
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        // Decode base64 content
        let decoded = BASE64.decode(response.content)?;
        let content = String::from_utf8(decoded)?;

        Ok(content)
    }

    pub async fn create_file(
        &self,
        project_id: u64,
        file_path: &str,
        branch: &str,
        commit_message: &str,
        content: &str,
    ) -> Result<()> {
        let encoded_path = urlencoding::encode(file_path);
        let url = format!(
            "{}/projects/{}/repository/files/{}",
            self.client.api_url(),
            project_id,
            encoded_path
        );

        debug!(
            "Creating file: {} in project {} branch {}",
            file_path, project_id, branch
        );

        // Encode content as base64
        let encoded_content = BASE64.encode(content.as_bytes());

        let body = json!({
            "branch": branch,
            "content": encoded_content,
            "commit_message": commit_message,
        });

        self.client
            .http_client()
            .post(&url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn update_file(
        &self,
        project_id: u64,
        file_path: &str,
        branch: &str,
        commit_message: &str,
        content: &str,
    ) -> Result<()> {
        let encoded_path = urlencoding::encode(file_path);
        let url = format!(
            "{}/projects/{}/repository/files/{}",
            self.client.api_url(),
            project_id,
            encoded_path
        );

        debug!(
            "Updating file: {} in project {} branch {}",
            file_path, project_id, branch
        );

        // Encode content as base64
        let encoded_content = BASE64.encode(content.as_bytes());

        let body = json!({
            "branch": branch,
            "content": encoded_content,
            "commit_message": commit_message,
        });

        self.client
            .http_client()
            .put(&url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}
