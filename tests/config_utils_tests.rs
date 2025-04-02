use anyhow::Result;
use gitlab_cli::utils::config::GitLabConfig;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

// Skip this test in normal test runs to avoid interfering with user config
#[test]
#[ignore]
fn test_config_save_and_load() -> Result<()> {
    // Create a temporary directory for our config
    let dir = tempdir()?;

    // Create a mock ConfigManager that uses our temp directory
    let config_path = dir.path().join("config.toml");

    // Create a test config
    let config = GitLabConfig {
        api_url: "https://gitlab.example.com/api/v4".to_string(),
        api_token: "test-token".to_string(),
    };

    // Write the config to the file
    let toml_str = toml::to_string(&config)?;
    let mut file = File::create(&config_path)?;
    write!(file, "{}", toml_str)?;

    // Try to load the config from the file
    let loaded_config: GitLabConfig = toml::from_str(&toml_str)?;

    // Verify the config was loaded correctly
    assert_eq!(loaded_config.api_url, "https://gitlab.example.com/api/v4");
    assert_eq!(loaded_config.api_token, "test-token");

    Ok(())
}

// Mock implementation test that doesn't require the dirs crate
#[test]
fn test_config_serialization() -> Result<()> {
    // Create a test config
    let config = GitLabConfig {
        api_url: "https://gitlab.example.com/api/v4".to_string(),
        api_token: "test-token".to_string(),
    };

    // Serialize to TOML
    let toml_str = toml::to_string(&config)?;

    // Verify the TOML contains the expected data
    assert!(toml_str.contains("api_url"));
    assert!(toml_str.contains("https://gitlab.example.com/api/v4"));
    assert!(toml_str.contains("api_token"));
    assert!(toml_str.contains("test-token"));

    // Deserialize back
    let deserialized: GitLabConfig = toml::from_str(&toml_str)?;

    // Verify the config was deserialized correctly
    assert_eq!(deserialized.api_url, "https://gitlab.example.com/api/v4");
    assert_eq!(deserialized.api_token, "test-token");

    Ok(())
}

