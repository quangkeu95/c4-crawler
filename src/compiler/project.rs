use std::{
    env,
    path::{Path, PathBuf},
};

use crate::types::RepoUri;

#[derive(Debug, Clone)]
pub enum ProjectType {
    Foundry,
    Hardhat,
    Truffle,
    Unknown,
}

impl ProjectType {
    pub fn from_repo_dir<P>(repo_dir: P) -> Self
    where
        P: AsRef<Path>,
    {
        let repo_dir = repo_dir.as_ref().to_path_buf();

        if repo_dir.join("foundry.toml").exists() {
            return Self::Foundry;
        };

        if repo_dir.join("hardhat.config.js").exists()
            || repo_dir.join("hardhat.config.ts").exists()
        {
            return Self::Hardhat;
        };

        if repo_dir.join("truffle-config.js").exists() {
            return Self::Truffle;
        };
        Self::Unknown
    }
}

pub fn project_dir_from_uri(repo_uri: &str) -> PathBuf {
    let dir_name = RepoUri::from(repo_uri.to_string());
    let dir_name = dir_name.to_dir_name().unwrap();
    let dir_path = PathBuf::from(env::current_dir().unwrap())
        .join("contests")
        .join(dir_name);

    dir_path
}
