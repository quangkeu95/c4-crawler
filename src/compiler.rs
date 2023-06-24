use anyhow::anyhow;
use ethers::types::Bytes;
use rr_logging::{error, info};
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

// pub fn compile<P>(repo_path: P) -> Result<ProjectCompileOutput, AppError>
// where
//     P: AsRef<Path>,
// {
//     let repo_path = repo_path.as_ref().to_path_buf();
//     let foundry_config_path = repo_path.join("foundry.toml");

//     if !foundry_config_path.exists() {
//         return Err(AppError::UnknownError(anyhow!("foundry.toml not found")));
//     }
//     // trying to parse foundry.toml
//     let project_path_config = parse_foundry_config(foundry_config_path)?;
//     info!("Project path config {:?}", project_path_config);

//     let project = Project::builder().paths(project_path_config).build()?;

//     project.compile().map_err(|e| AppError::from(e))
// }

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

pub fn find_all_contracts<P>(repo_dir: P) -> Result<Vec<Contract>, AppError>
where
    P: AsRef<Path>,
{
    let repo_dir = repo_dir.as_ref().to_path_buf();
    let foundry_config_path = repo_dir.join("foundry.toml");

    if !foundry_config_path.exists() {
        return Err(AppError::UnknownError(anyhow!("foundry.toml not found")));
    }

    // trying to build
    forge_build(&repo_dir)?;

    let project = Project::builder()
        .paths(
            ProjectPathsConfig::builder()
                .root(repo_dir)
                .build()
                .unwrap(),
        )
        .build()?;

    let contracts = ContractResolver::get_contracts_from_project(&project).unwrap();
    Ok(contracts)
}

fn forge_build<P>(repo_dir: P) -> Result<(), AppError>
where
    P: AsRef<Path>,
{
    let repo_dir = repo_dir.as_ref().to_path_buf();
    info!("Running forge build in directory {:?}", repo_dir);

    let output = Command::new("forge")
        .args(&["build"])
        .current_dir(repo_dir.clone())
        .output()
        .expect("Failed to execute 'git clone' command");

    if output.status.success() {
        info!(
            "Forge build successfully in directory {:?}",
            repo_dir.clone()
        );
    } else {
        // repo existed

        let error_message = String::from_utf8_lossy(&output.stderr);
        error!(
            "Error forge build in directory {:?}, error: {}",
            repo_dir, error_message
        );

        return Err(AppError::UnknownError(anyhow!(error_message.to_string())));
    }
    Ok(())
}

fn parse_foundry_config<P>(file_path: P) -> Result<ProjectPathsConfig, AppError>
where
    P: AsRef<Path>,
{
    // parsing foundry.toml
    let file_path = file_path.as_ref().to_path_buf();
    info!("Parsing foundry config at {:?}", file_path);

    let parent_dir = file_path
        .parent()
        .expect(format!("Missing parent dir for file {:?}", &file_path).as_str())
        .to_path_buf();
    let file_content =
        fs::read_to_string(file_path).map_err(|e| AppError::UnknownError(anyhow!(e)))?;
    let parsed_toml: FoundryConfig =
        toml::from_str(&file_content).map_err(|e| AppError::UnknownError(anyhow!(e)))?;

    let mut project_path_config = ProjectPathsConfig::builder().root(parent_dir.clone());

    if let Some(src_config) = parsed_toml.profile.default.src {
        let src_path = parent_dir.join(src_config);
        project_path_config = project_path_config.sources(src_path);
    }

    if let Some(libs_config) = parsed_toml.profile.default.libs {
        let libs_path: Vec<PathBuf> = libs_config
            .into_iter()
            .map(|p| parent_dir.join(p))
            .collect();
        project_path_config = project_path_config.libs(libs_path);
    }

    // // parsing remappings.txt
    // let remappings_file_path = parent_dir.join("remappings.txt");
    // if remappings_file_path.exists() {
    //     info!("Parsing remappings at {:?}", remappings_file_path);
    //     let remappings = parse_remappings(remappings_file_path)?;
    //     if remappings.len() > 0 {
    //         project_path_config = project_path_config.remappings(remappings);
    //     }
    // }
    project_path_config.build().map_err(|e| AppError::from(e))
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
