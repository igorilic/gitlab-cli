use anyhow::Result;
use serde_json::json;
use tracing::debug;

use super::client::GitLabClient;
use crate::models::user::{AccessLevel, User};

pub struct UsersApi<'a> {
    client: &'a GitLabClient,
}

impl<'a> UsersApi<'a> {
    pub fn new(client: &'a GitLabClient) -> Self {
        Self { client }
    }

    pub async fn get_by_id(&self, id: u64) -> Result<User> {
        let url = format!("{}/users/{}", self.client.api_url(), id);

        debug!("Fetching user by ID: {}", id);

        let response = self
            .client
            .http_client()
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<User>()
            .await?;

        Ok(response)
    }

    pub async fn get_by_username(&self, username: &str) -> Result<User> {
        let url = format!("{}/users?username={}", self.client.api_url(), username);

        debug!("Fetching user by username: {}", username);

        let users: Vec<User> = self
            .client
            .http_client()
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        if let Some(user) = users.into_iter().next() {
            Ok(user)
        } else {
            anyhow::bail!("User not found: {}", username)
        }
    }

    pub async fn add_to_project(
        &self,
        user_id: u64,
        project_id: u64,
        access_level: AccessLevel,
    ) -> Result<()> {
        // First, try the members endpoint (works for self-managed GitLab instances)
        let members_url = format!("{}/projects/{}/members", self.client.api_url(), project_id);

        debug!(
            "Attempting to add user {} to project {} with access level {:?} using members endpoint",
            user_id, project_id, access_level
        );

        let body = json!({
            "user_id": user_id,
            "access_level": access_level.as_u64(),
        });

        let members_response = self
            .client
            .http_client()
            .post(&members_url)
            .json(&body)
            .send()
            .await;

        match members_response {
            Ok(response) => {
                if response.status().is_success() {
                    debug!("Successfully added user to project using members endpoint");
                    return Ok(());
                }

                // If members endpoint failed, try the invitations endpoint
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                debug!("Members endpoint failed with error: {}", error_text);

                // Try invitations endpoint as fallback (required for GitLab.com)
                let invitations_url = format!(
                    "{}/projects/{}/invitations",
                    self.client.api_url(),
                    project_id
                );

                debug!(
                    "Attempting to add user {} to project {} with access level {:?} using invitations endpoint",
                    user_id, project_id, access_level
                );

                // Invitations endpoint has a different payload structure
                let invitation_body = json!({
                    "user_id": user_id.to_string(), // API accepts both integer and string
                    "access_level": access_level.as_u64(),
                });

                let invitation_response = self
                    .client
                    .http_client()
                    .post(&invitations_url)
                    .json(&invitation_body)
                    .send()
                    .await?;

                if invitation_response.status().is_success() {
                    debug!("Successfully added user to project using invitations endpoint");
                    return Ok(());
                }

                let invitation_error = invitation_response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                anyhow::bail!(
                    "Failed to add user to project. Members endpoint error: {}. Invitations endpoint error: {}",
                    error_text,
                    invitation_error
                );
            }
            Err(e) => {
                debug!("Members endpoint request failed: {}", e);

                // Try invitations endpoint
                let invitations_url = format!(
                    "{}/projects/{}/invitations",
                    self.client.api_url(),
                    project_id
                );

                debug!(
                    "Attempting to add user {} to project {} with access level {:?} using invitations endpoint",
                    user_id, project_id, access_level
                );

                // Invitations endpoint has a different payload structure
                let invitation_body = json!({
                    "user_id": user_id.to_string(), // API accepts both integer and string
                    "access_level": access_level.as_u64(),
                });

                let response = self
                    .client
                    .http_client()
                    .post(&invitations_url)
                    .json(&invitation_body)
                    .send()
                    .await?;

                if response.status().is_success() {
                    debug!("Successfully added user to project using invitations endpoint");
                    return Ok(());
                }

                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                anyhow::bail!("Failed to add user to project: {}", error_text);
            }
        }
    }

    pub async fn remove_from_project(&self, user_id: u64, project_id: u64) -> Result<()> {
        let url = format!(
            "{}/projects/{}/members/{}",
            self.client.api_url(),
            project_id,
            user_id
        );

        debug!("Removing user {} from project {}", user_id, project_id);

        self.client
            .http_client()
            .delete(&url)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}
