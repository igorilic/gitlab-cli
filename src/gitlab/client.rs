use reqwest::{Client, header};
use tracing::debug;

use super::{files::FilesApi, projects::ProjectsApi, users::UsersApi};

pub struct GitLabClient {
    api_url: String,
    api_token: String,
    http_client: Client,
}

impl GitLabClient {
    pub fn new(api_url: &str, api_token: &str) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "PRIVATE-TOKEN",
            header::HeaderValue::from_str(api_token).expect("Invalid API token"),
        );

        let http_client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client");

        debug!("Created GitLab client for {}", api_url);

        Self {
            api_url: api_url.to_string(),
            api_token: api_token.to_string(),
            http_client,
        }
    }

    pub fn projects(&self) -> ProjectsApi {
        ProjectsApi::new(self)
    }

    pub fn users(&self) -> UsersApi {
        UsersApi::new(self)
    }

    pub fn files(&self) -> FilesApi {
        FilesApi::new(self)
    }

    pub fn api_url(&self) -> &str {
        &self.api_url
    }

    pub fn http_client(&self) -> &Client {
        &self.http_client
    }
}
