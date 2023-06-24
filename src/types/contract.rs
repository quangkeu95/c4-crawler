use crate::errors::{AppError, ContractError};
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use ethers::types::Bytes;
use ethers_solc::{
    artifacts::NodeType,
    cache::{CacheEntry, SolFilesCache},
    ArtifactOutput, ConfigurableArtifacts, ConfigurableContractArtifact, Project,
};
use rayon::prelude::*;
use rr_logging::info;
use semver::Version;
use serde::Deserialize;
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt::Display,
    path::{Path, PathBuf},
    sync::Arc,
};
use walkdir::WalkDir;

#[derive(Debug, Clone, Builder)]
pub struct Contract {
    pub name: String,
    // pub bytecode: ContractBytecode,
    pub kind: ContractKind,
    pub version: Version,
    #[builder(default)]
    pub imported_contracts: Vec<ContractFromArtifact>,
}

impl PartialOrd for Contract {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if matches!(self.kind, ContractKind::Interface)
            && matches!(other.kind, ContractKind::Contract(_))
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
        self.kind == other.kind && self.name == other.name
    }
}

#[derive(Debug, Clone)]
pub struct ContractFromArtifact {
    pub name: String,
    pub kind: ContractKind,
    pub artifact_path: PathBuf,
}

impl PartialOrd for ContractFromArtifact {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if matches!(self.kind, ContractKind::Interface)
            && matches!(other.kind, ContractKind::Contract(_))
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

impl PartialEq for ContractFromArtifact {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.name == other.name
    }
}

#[derive(Clone, Default, PartialEq, PartialOrd, Ord, Eq)]
pub struct ContractBytecode(String);

impl From<String> for ContractBytecode {
    fn from(value: String) -> Self {
        ContractBytecode(value)
    }
}

impl Display for ContractBytecode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Debug for ContractBytecode {
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
    Contract(ContractBytecode),
}

impl From<ContractBytecode> for ContractKind {
    fn from(value: ContractBytecode) -> Self {
        if value.0 == "0x" {
            Self::Interface
        } else {
            Self::Contract(value)
        }
    }
}
