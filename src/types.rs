use serde::Deserialize;
use std::{cmp::Ordering, fmt::Display, path::PathBuf};

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

#[derive(Debug, Clone)]
pub struct Contract {
    pub name: String,
    pub bytecode: ContractBytecode,
    pub kind: ContractKind,
}

impl Contract {
    pub fn contract_kind(bytecode: &str) -> ContractKind {
        if bytecode == "0x" {
            ContractKind::Interface
        } else {
            ContractKind::Contract
        }
    }
}

impl PartialOrd for Contract {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if matches!(self.kind, ContractKind::Interface)
            && matches!(other.kind, ContractKind::Contract)
        {
            Some(Ordering::Less)
        } else if matches!(self.kind, ContractKind::Interface)
            && matches!(other.kind, ContractKind::Interface)
        {
            Some(Ordering::Equal)
        } else {
            Some(Ordering::Greater)
        }
    }
}

impl PartialEq for Contract {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.name == other.name && self.bytecode == other.bytecode
    }
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Ord, Eq)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum ContractKind {
    Interface,
    Contract,
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
