use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: u64,
    pub path_with_namespace: String,
    pub name: String,
    pub description: Option<String>,
    pub default_branch: Option<String>,
    pub visibility: String,
    pub web_url: String,
    #[serde(default)]
    pub topics: Vec<String>,
}
