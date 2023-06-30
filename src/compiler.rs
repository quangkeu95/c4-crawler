use anyhow::anyhow;
use ethers::types::Bytes;
use rr_logging::{error, info, instrument, tracing};
use std::{
    cmp::Ordering,
    env,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    errors::AppError,
    types::{Contest, Contract, ContractBytecode, ContractKind, FoundryConfig, RepoUri},
};
use ethers_solc::{
    buildinfo::BuildInfo, output::ProjectCompileOutput, project_util::TempProject,
    remappings::Remapping, Artifact, ArtifactOutput, ConfigurableArtifacts, Project,
    ProjectPathsConfig,
};
use walkdir::WalkDir;

mod contract;
pub use contract::*;
mod project;
pub use project::*;

/// Clone the repo if it's not cloned, else pull from branch main
pub fn clone_or_pull_repo(repo_uri: &str) -> Result<PathBuf, AppError> {
    // create directory contains the contest repo
    let repo_dir = project_dir_from_uri(repo_uri);

    info!("Creating directory if not existed: {:?}", repo_dir);
    fs::create_dir_all(repo_dir.clone()).map_err(|e| AppError::UnknownError(anyhow!(e)))?;

    // TODO: pull the repo if the repo is existed, for now we just clear the repo and reclone
    if is_directory_empty(&repo_dir) {
        // Execute the `git clone` command
        let output = Command::new("git")
            .args(&["clone", &repo_uri, repo_dir.as_os_str().to_str().unwrap()])
            .output()
            .expect("Failed to execute 'git clone' command");

        if output.status.success() {
            info!("Repository {:?} cloned successfully!", repo_uri);
        } else {
            // repo existed

            let error_message = String::from_utf8_lossy(&output.stderr);
            error!(
                "Error cloning repository {:?}, error: {}",
                repo_uri, error_message
            );

            return Err(AppError::UnknownError(anyhow!(error_message.to_string())));
        }
    }

    Ok(repo_dir)
}

#[instrument(skip_all)]
pub fn find_all_contracts<P>(repo_dir: P) -> Result<Vec<Contract>, AppError>
where
    P: AsRef<Path>,
{
    let projects = ProjectResolver::parse(repo_dir)?;

    let contracts = projects
        .iter()
        .filter_map(|project| ContractResolver::get_contracts_from_project(&project).ok())
        .flatten()
        .collect::<Vec<Contract>>();
    Ok(contracts)
}

fn parse_remappings<P>(file_path: P) -> Result<Vec<Remapping>, AppError>
where
    P: AsRef<Path>,
{
    let mut result: Vec<Remapping> = vec![];

    let file_path = file_path.as_ref().to_path_buf();
    let parent_dir = file_path
        .parent()
        .expect(format!("Missing parent dir for file {:?}", &file_path).as_str())
        .to_path_buf();

    let file = File::open(file_path).map_err(|e| AppError::UnknownError(anyhow!(e)))?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        let line_split: Vec<&str> = line.split("=").collect();
        if line_split.len() < 2 {
            continue;
        }

        let name = line_split[0];
        let mapping_path = parent_dir
            .join(line_split[1])
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string();

        result.push(Remapping {
            name: name.to_string(),
            path: mapping_path,
        });
    }

    Ok(result)
}

fn is_directory_empty<P>(dir_path: P) -> bool
where
    P: AsRef<Path>,
{
    if let Ok(entries) = fs::read_dir(dir_path) {
        return entries.count() == 0;
    }
    false
}

#[cfg(test)]
mod test {
    use super::find_all_contracts;
    use claims::*;
    use std::path::PathBuf;

    #[test]
    fn test_find_all_contracts() {
        let repo_dir = PathBuf::from("contests/2023-05-maia");

        let contracts = find_all_contracts(&repo_dir).unwrap();

        assert_gt!(contracts.len(), 0);
        // println!("contracts {:#?}", contracts);
    }
}
