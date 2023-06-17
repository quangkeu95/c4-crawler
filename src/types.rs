use serde::Deserialize;
use std::path::PathBuf;

use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Contest {
    pub name: String,
    pub description: String,
    pub uri: String,
    pub repo_uri: Option<String>,
    pub status: ContestStatus,
    // start_date: DateTime<Utc>,
    // end_date: DateTime<Utc>,
    // reward: String,
    pub contracts: Vec<Contract>,
}

#[derive(Debug)]
pub enum ContestStatus {
    Ongoing,
    Upcoming,
}

#[derive(Debug, Clone, Default)]
pub struct Contract {
    pub name: String,
    pub bytecode: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FoundryConfig {
    pub profile: FoundryConfigProfile,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FoundryConfigProfile {
    pub default: FoundryConfigProfileDefault,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FoundryConfigProfileDefault {
    pub src: Option<String>,
    pub libs: Option<Vec<String>>,
}
