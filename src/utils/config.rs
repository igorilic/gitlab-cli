use anyhow::{Context, Result};
use config::{Config, File};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::debug;

#[derive(Debug, Deserialize, Serialize)]
pub struct GitLabConfig {
    pub api_url: String,
    pub api_token: String,
}

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<ConfigManager> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("gitlab-bulk-cli");

        std::fs::create_dir_all(&config_dir)
            .with_context(|| format!("Failed to create config directory: {:?}", config_dir))?;

        let config_path = config_dir.join("config.toml");

        debug!("Using config file: {:?}", config_path);

        Ok(Self { config_path })
    }

    pub fn load(&self) -> Result<GitLabConfig> {
        if !self.config_path.exists() {
            anyhow::bail!("Config file does not exist: {:?}", self.config_path);
        }

        let config = Config::builder()
            .add_source(File::from(self.config_path.clone()))
            .build()
            .with_context(|| format!("Failed to load config file: {:?}", self.config_path))?;

        let gitlab_config = config
            .try_deserialize::<GitLabConfig>()
            .with_context(|| format!("Failed to parse config file: {:?}", self.config_path))?;

        Ok(gitlab_config)
    }

    pub fn save(&self, config: &GitLabConfig) -> Result<()> {
        let toml = toml::to_string(config).with_context(|| "Failed to serialize config")?;

        std::fs::write(&self.config_path, toml)
            .with_context(|| format!("Failed to write config file: {:?}", self.config_path))?;

        debug!("Saved config to: {:?}", self.config_path);

        Ok(())
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }
}
