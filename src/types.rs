use serde::Deserialize;
use std::{fmt::Display, path::PathBuf};

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
    pub bytecode: ContractBytecode,
}

#[derive(Debug, Clone, Default)]
pub struct ContractBytecode(String);

impl From<String> for ContractBytecode {
    fn from(value: String) -> Self {
        ContractBytecode(value)
    }
}

impl Display for ContractBytecode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = &self.0;
        let length_threshold = 20;
        if data.len() <= length_threshold {
            return write!(f, "{}", data);
        }

        let start_chars = &data[..10];
        let end_chars = &data[data.len() - 10..];

        write!(f, "{}..{}", start_chars, end_chars)
    }
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
