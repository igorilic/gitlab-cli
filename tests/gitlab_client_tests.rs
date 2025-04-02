use gitlab_cli::gitlab::client::GitLabClient;

#[test]
fn test_client_creation() {
    let api_url = "https://gitlab.example.com/api/v4";
    let api_token = "test-token";

    let client = GitLabClient::new(api_url, api_token);

    assert_eq!(client.api_url(), api_url);

    // Test API facades
    let _projects_api = client.projects();
    let _users_api = client.users();
    let _files_api = client.files();
}
