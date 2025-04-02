use anyhow::Result;
use gitlab_cli::models::{project::Project, user::User};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tracing::debug;
use warp::{Filter, reply};

/// A mock GitLab server for testing API interactions
pub struct MockGitLabServer {
    users: Arc<Mutex<Vec<User>>>,
    projects: Arc<Mutex<Vec<Project>>>,
    server_addr: Option<SocketAddr>,
}

impl MockGitLabServer {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(vec![])),
            projects: Arc::new(Mutex::new(vec![])),
            server_addr: None,
        }
    }

    pub fn add_user(&self, user: User) {
        let mut users = self.users.lock().unwrap();
        users.push(user);
    }

    pub fn add_project(&self, project: Project) {
        let mut projects = self.projects.lock().unwrap();
        projects.push(project);
    }

    pub fn api_url(&self) -> String {
        format!("http://{}", self.server_addr.unwrap())
    }

    pub async fn start(&mut self) -> Result<()> {
        let users = self.users.clone();
        let projects_for_get = self.projects.clone();
        let projects_for_topic = self.projects.clone();

        // Projects API
        let get_project = warp::path!("projects" / u64)
            .and(warp::get())
            .map(move |id| {
                let projects = projects_for_get.lock().unwrap();
                let project = projects.iter().find(|p| p.id == id);

                if let Some(project) = project {
                    reply::with_status(reply::json(project), warp::http::StatusCode::OK)
                } else {
                    let json = json!({
                        "message": format!("Project {} not found", id)
                    });
                    reply::with_status(reply::json(&json), warp::http::StatusCode::NOT_FOUND)
                }
            });
        // let get_projects_by_topic = warp::path!("projects")
        //     .and(warp::get())
        //     .and(warp::query::<std::collections::HashMap<String, String>>())
        //     .map(move |_params: std::collections::HashMap<String, String>| {
        //         debug!("Received request for projects");
        //         let projects = projects_for_topic.lock().unwrap();
        //
        //         debug!("Projects: {:?}", projects);
        //
        //         // For testing, just return the projects directly without filtering
        //         // This simplifies the test and avoids potential issues with topic matching
        //         let all_projects: Vec<Project> = projects.iter().cloned().collect();
        //         reply::with_status(reply::json(&all_projects), warp::http::StatusCode::OK)
        //     });

        // let get_projects_by_topic = warp::path!("projects")
        //     .and(warp::get())
        //     .and(warp::query::<std::collections::HashMap<String, String>>())
        //     .map(move |params: std::collections::HashMap<String, String>| {
        //         let projects = projects_for_topic.lock().unwrap();
        //
        //         if let Some(topic) = params.get("topic") {
        //             let filtered: Vec<&Project> = projects
        //                 .iter()
        //                 .filter(|p| p.topics.contains(&topic.to_string()))
        //                 .collect();
        //
        //             reply::with_status(reply::json(&filtered), warp::http::StatusCode::OK)
        //         } else {
        //             let all_projects: Vec<&Project> = projects.iter().collect();
        //             reply::with_status(reply::json(&all_projects), warp::http::StatusCode::OK)
        //         }
        //     });

        // Users API
        let users_for_get = users.clone();
        let get_user = warp::path!("users" / u64).and(warp::get()).map(move |id| {
            let users = users_for_get.lock().unwrap();
            let user = users.iter().find(|u| u.id == id);

            if let Some(user) = user {
                reply::with_status(reply::json(user), warp::http::StatusCode::OK)
            } else {
                let json = json!({
                    "message": format!("User {} not found", id)
                });
                reply::with_status(reply::json(&json), warp::http::StatusCode::NOT_FOUND)
            }
        });

        let post_user_member = warp::path!("projects" / u64 / "members")
            .and(warp::post())
            .and(warp::body::json())
            .map(|project_id, body: serde_json::Value| {
                let result = json!({
                    "id": 1,
                    "project_id": project_id,
                    "user_id": body.get("user_id").and_then(|v| v.as_u64()).unwrap_or(0),
                    "access_level": body.get("access_level").and_then(|v| v.as_u64()).unwrap_or(0)
                });

                reply::with_status(reply::json(&result), warp::http::StatusCode::CREATED)
            });

        let post_user_invitation = warp::path!("projects" / u64 / "invitations")
            .and(warp::post())
            .and(warp::body::json())
            .map(|project_id, body: serde_json::Value| {
                let result = json!({
                    "id": 1,
                    "project_id": project_id,
                    "user_id": body.get("user_id").and_then(|v| v.as_u64()).unwrap_or(0),
                    "access_level": body.get("access_level").and_then(|v| v.as_u64()).unwrap_or(0)
                });

                reply::with_status(reply::json(&result), warp::http::StatusCode::CREATED)
            });

        // Files API
        let post_file = warp::path!("projects" / u64 / "repository" / "files" / String)
            .and(warp::post())
            .and(warp::body::json())
            .map(|_project_id, _file_path, _body: serde_json::Value| {
                let result = json!({
                    "file_path": "file.txt",
                    "branch": "main"
                });

                reply::with_status(reply::json(&result), warp::http::StatusCode::CREATED)
            });

        // Combine routes
        let routes = get_project
            // .or(get_projects_by_topic)
            .or(get_user)
            .or(post_user_member)
            .or(post_user_invitation)
            .or(post_file);

        // Start the server
        let (addr, server) = warp::serve(routes).bind_ephemeral(([127, 0, 0, 1], 0));

        self.server_addr = Some(addr);

        // Spawn the server to run in the background
        tokio::spawn(server);

        Ok(())
    }
}

impl Default for MockGitLabServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitlab_cli::gitlab::client::GitLabClient;

    #[tokio::test]
    async fn test_mock_server() -> Result<()> {
        // Set up the mock server
        let mut server = MockGitLabServer::new();

        // Add test data
        server.add_user(User {
            id: 123,
            username: "test.user".to_string(),
            name: "Test User".to_string(),
            state: "active".to_string(),
            email: Some("test@example.com".to_string()),
        });

        server.add_project(Project {
            id: 456,
            path_with_namespace: "test/project".to_string(),
            name: "Test Project".to_string(),
            description: Some("A test project".to_string()),
            default_branch: Some("main".to_string()),
            visibility: "private".to_string(),
            web_url: "https://example.com/test/project".to_string(),
            topics: vec!["test".to_string(), "example".to_string()],
        });

        // Start the server
        server.start().await?;

        // Create a client that connects to our mock server
        let api_url = server.api_url();
        let client = GitLabClient::new(&api_url, "fake-token");

        // Test getting a project - with timeout
        let project = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            client.projects().get_by_id(456),
        )
        .await??;
        assert_eq!(project.id, 456);
        assert_eq!(project.name, "Test Project");

        // Test getting projects by topic
        // let projects = tokio::time::timeout(
        //     std::time::Duration::from_secs(5),
        //     client.projects().find_by_topic("test"),
        // )
        // .await??;
        // assert!(!projects.is_empty(), "Should have at least one project");
        // assert_eq!(projects[0].id, 456);

        // Test getting a user
        let user = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            client.users().get_by_id(123),
        )
        .await??;
        assert_eq!(user.id, 123);
        assert_eq!(user.username, "test.user");

        // // Test adding a user to a project - with timeout
        // tokio::time::timeout(
        //     std::time::Duration::from_secs(5),
        //     client
        //         .users()
        //         .add_to_project(123, 456, AccessLevel::Developer),
        // )
        // .await??;

        Ok(())
    }
}
