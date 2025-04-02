# GitLab Bulk CLI

A powerful command-line tool for managing GitLab users, topics, and files across multiple repositories.

## Features

- **User Management:** Add or remove users with specific roles across multiple projects
- **Topic Management:** Add, remove, or list topics for projects
- **File Management:** Add or update files across multiple repositories
- **Flexible Selection:** Select projects by ID, path, or topic

## Installation

### From Source

```bash
git clone https://github.com/igorilic/gitlab-cli.git
cd gitlab-bulk-cli
cargo build --release
```

The binary will be available at `target/release/gitlab-bulk`.

## Usage

### Environment Variables

Set your GitLab API credentials:

```bash
export GITLAB_API_URL="https://gitlab.your-company.com/api/v4"
export GITLAB_API_TOKEN="your-api-token"
```

### User Management

Add users to projects:

```bash
# Add users from CSV file to projects with a specific topic
gitlab-bulk users add --user-file users.csv --topic backend --role maintainer

# Add specific users by ID/username to specific projects
gitlab-bulk users add --user-ids john.doe,jane.smith --project-ids 123,456 --role developer
```

Remove users from projects:

```bash
# Remove users in CSV file from all projects with specific topic
gitlab-bulk users remove --user-file users.csv --topic deprecated

# Remove specific users from specific projects
gitlab-bulk users remove --user-ids john.doe,jane.smith --project-ids 123,456
```

### Topic Management

Add topics to projects:

```bash
# Add topics to projects with a specific filter topic
gitlab-bulk topics add team:backend,env:prod --filter-topic service

# Add topics to specific projects
gitlab-bulk topics add team:frontend,env:staging --project-ids 123,456
```

Remove topics from projects:

```bash
# Remove topics from projects
gitlab-bulk topics remove env:staging --project-ids 123,456
```

List topics for projects:

```bash
# List topics for projects with a specific topic
gitlab-bulk topics list --filter-topic team:backend

# List topics for specific projects
gitlab-bulk topics list --project-ids 123,456
```

### File Management

Update files across projects:

```bash
# Add or update a config file in all "services" projects
gitlab-bulk file update --file-path ./local/config.yml --target-path config/config.yml \
  --topic services --commit-message "Update config file"

# Update a file with string replacements
gitlab-bulk file update --file-path ./Dockerfile --target-path Dockerfile \
  --project-ids 123,456 --changes "FROM python:3.8:FROM python:3.9;EXPOSE 8000:EXPOSE 8080" \
  --commit-message "Update Docker configuration"
```

## CSV File Formats

### Users CSV

```csv
id,username,name,email
123,john.doe,John Doe,john.doe@example.com
456,jane.smith,Jane Smith,jane.smith@example.com
```

### Projects CSV

```csv
id,path_with_namespace,name,description,default_branch,topics
123,group/project-a,Project A,Example project,main,backend,service
456,group/project-b,Project B,Another project,master,frontend
```

## Development

### Running Tests

Run the standard tests:

```bash
cargo test
```

Run with coverage report:

```bash
# Install cargo-tarpaulin (if not already installed)
cargo install cargo-tarpaulin

# Run tests with coverage
cargo tarpaulin
```

### Running Integration Tests

To run integration tests (requires a GitLab instance):

```bash
export GITLAB_API_URL="https://gitlab.your-company.com/api/v4"
export GITLAB_API_TOKEN="your-api-token"
cargo test -- --ignored
```

## License

MIT
