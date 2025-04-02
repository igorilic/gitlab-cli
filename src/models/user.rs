use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub name: String,
    pub state: String,
    #[serde(default)]
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum AccessLevel {
    NoAccess,
    MinimalAccess,
    Guest,
    Planner,
    Reporter,
    Developer,
    Maintainer,
    Owner,
}

impl AccessLevel {
    pub fn as_u64(&self) -> u64 {
        match self {
            Self::NoAccess => 0,
            Self::MinimalAccess => 5,
            Self::Guest => 10,
            Self::Planner => 15,
            Self::Reporter => 20,
            Self::Developer => 30,
            Self::Maintainer => 40,
            Self::Owner => 50,
        }
    }
}

impl FromStr for AccessLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "noaccess" | "no_access" | "no-access" | "0" => Ok(Self::NoAccess),
            "minimalaccess" | "minimal_access" | "minimal-access" | "5" => Ok(Self::MinimalAccess),
            "guest" | "10" => Ok(Self::Guest),
            "planner" | "15" => Ok(Self::Planner),
            "reporter" | "20" => Ok(Self::Reporter),
            "developer" | "30" => Ok(Self::Developer),
            "maintainer" | "40" => Ok(Self::Maintainer),
            "owner" | "50" => Ok(Self::Owner),
            _ => Err(format!("Invalid access level: {}", s)),
        }
    }
}

impl fmt::Display for AccessLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoAccess => write!(f, "No Access"),
            Self::MinimalAccess => write!(f, "Minimal Access"),
            Self::Guest => write!(f, "Guest"),
            Self::Planner => write!(f, "Planner"),
            Self::Reporter => write!(f, "Reporter"),
            Self::Developer => write!(f, "Developer"),
            Self::Maintainer => write!(f, "Maintainer"),
            Self::Owner => write!(f, "Owner"),
        }
    }
}
