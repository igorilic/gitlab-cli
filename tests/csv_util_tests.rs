use anyhow::Result;
use gitlab_cli::utils::csv::CsvReader;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_read_users_from_csv() -> Result<()> {
    // Create a temporary directory
    let dir = tempdir()?;
    let file_path = dir.path().join("users.csv");

    // Write test data to the temporary file
    let mut file = File::create(&file_path)?;
    writeln!(file, "id,username,name,email")?;
    writeln!(file, "123,john.doe,John Doe,john.doe@example.com")?;
    writeln!(file, "456,jane.smith,Jane Smith,jane.smith@example.com")?;

    // Create CSV reader and parse the file
    let reader = CsvReader::new(&file_path)?;
    let users = reader.read_users()?;

    // Verify the results
    assert_eq!(users.len(), 2);

    assert_eq!(users[0].id, 123);
    assert_eq!(users[0].username, "john.doe");
    assert_eq!(users[0].name, "John Doe");
    assert_eq!(users[0].email, Some("john.doe@example.com".to_string()));

    assert_eq!(users[1].id, 456);
    assert_eq!(users[1].username, "jane.smith");
    assert_eq!(users[1].name, "Jane Smith");
    assert_eq!(users[1].email, Some("jane.smith@example.com".to_string()));

    Ok(())
}

#[test]
fn test_read_projects_from_csv() -> Result<()> {
    // Create a temporary directory
    let dir = tempdir()?;
    let file_path = dir.path().join("projects.csv");

    // Write test data to the temporary file
    let mut file = File::create(&file_path)?;
    writeln!(
        file,
        "id,path_with_namespace,name,description,default_branch,visibility,web_url,topics"
    )?;
    writeln!(
        file,
        "123,group/project-a,Project A,Example project A,main,private,https://example.com/group/project-a,\"backend,service\""
    )?;
    writeln!(
        file,
        "456,group/project-b,Project B,Example project B,master,public,https://example.com/group/project-b,frontend"
    )?;

    // Create CSV reader and parse the file
    let reader = CsvReader::new(&file_path)?;
    let projects = reader.read_projects()?;

    // Verify the results
    assert_eq!(projects.len(), 2);

    assert_eq!(projects[0].id, 123);
    assert_eq!(projects[0].path_with_namespace, "group/project-a");
    assert_eq!(projects[0].name, "Project A");
    assert_eq!(
        projects[0].description,
        Some("Example project A".to_string())
    );
    assert_eq!(projects[0].default_branch, Some("main".to_string()));
    assert_eq!(projects[0].visibility, "private");
    assert_eq!(projects[0].web_url, "https://example.com/group/project-a");
    assert_eq!(projects[0].topics, vec!["backend", "service"]);

    assert_eq!(projects[1].id, 456);
    assert_eq!(projects[1].path_with_namespace, "group/project-b");
    assert_eq!(projects[1].name, "Project B");
    assert_eq!(
        projects[1].description,
        Some("Example project B".to_string())
    );
    assert_eq!(projects[1].default_branch, Some("master".to_string()));
    assert_eq!(projects[1].visibility, "public");
    assert_eq!(projects[1].web_url, "https://example.com/group/project-b");
    assert_eq!(projects[1].topics, vec!["frontend"]);

    Ok(())
}

#[test]
fn test_read_users_from_invalid_file() {
    // Try to open a non-existent file
    let result = CsvReader::new("nonexistent.csv");
    assert!(result.is_err());

    // Error message should mention the file path
    let err = result.unwrap_err().to_string();
    assert!(err.contains("nonexistent.csv"));
}
