use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::tempdir;

// This test requires the binary to be built and a valid GitLab API token
// It's skipped by default unless the GITLAB_INTEGRATION_TEST environment variable is set
#[test]
#[ignore]
fn test_cli_help() -> Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_gitlab-cli"))
        .arg("--help")
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    let stdout = String::from_utf8(output.stdout)?;

    // Verify the help message contains expected commands
    assert!(stdout.contains("gitlab-bulk"));
    assert!(stdout.contains("user"));
    assert!(stdout.contains("file"));
    assert!(stdout.contains("topics"));

    Ok(())
}

#[test]
#[ignore]
fn test_cli_version() -> Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_gitlab-cli"))
        .arg("--version")
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    let stdout = String::from_utf8(output.stdout)?;

    // Verify the version is output
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));

    Ok(())
}

#[test]
#[ignore]
fn test_cli_users_list() -> Result<()> {
    // Only run if we have a valid GitLab API URL and token
    let api_url = std::env::var("GITLAB_API_URL").ok();
    let api_token = std::env::var("GITLAB_API_TOKEN").ok();

    if api_url.is_none() || api_token.is_none() {
        eprintln!("Skipping integration test: GITLAB_API_URL or GITLAB_API_TOKEN not set");
        return Ok(());
    }

    // Create a temporary file with project IDs
    let dir = tempdir()?;
    let file_path = dir.path().join("projects.csv");

    // Write test data to the temporary file
    let mut file = File::create(&file_path)?;
    writeln!(file, "id,path_with_namespace,name")?;
    writeln!(file, "123456,test/project,Test Project")?;

    // Run the command
    let output = Command::new(env!("CARGO_BIN_EXE_gitlab-cli"))
        .arg("--api-url")
        .arg(api_url.unwrap())
        .arg("--api-token")
        .arg(api_token.unwrap())
        .arg("topics")
        .arg("list")
        .arg("--project-file")
        .arg(file_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    // Check output - we expect an error because our test project ID is fake
    let stderr = String::from_utf8(output.stderr)?;

    // It should attempt to load the projects from the CSV file
    assert!(
        stderr.contains("Searching for projects")
            || stderr.contains("Loading projects")
            || stderr.contains("Project not found")
    );

    Ok(())
}
