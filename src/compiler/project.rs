use std::{
    collections::HashSet,
    env,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

use ethers_solc::{
    artifacts::ExpressionOrVariableDeclarationStatement, cache::SOLIDITY_FILES_CACHE_FILENAME,
    Project, ProjectPathsConfig,
};
use rr_logging::{error, info, instrument, tracing};
use walkdir::WalkDir;

use crate::{
    errors::AppError,
    types::{FoundryConfig, RepoUri},
};

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

        if repo_dir.join("hardhat.config.js").exists()
            || repo_dir.join("hardhat.config.ts").exists()
        {
            return Self::Hardhat;
        };

        if repo_dir.join("foundry.toml").exists() {
            return Self::Foundry;
        };

        if repo_dir.join("truffle-config.js").exists() {
            return Self::Truffle;
        };
        Self::Unknown
    }
}

/// Resolve project
pub struct ProjectResolver {}

impl ProjectResolver {
    #[instrument(skip_all)]
    pub fn parse<P>(repo_dir: P) -> Result<Vec<Project>, AppError>
    where
        P: AsRef<Path>,
    {
        let all_project_roots = find_all_project_roots(repo_dir.as_ref());
        let mut projects: Vec<Project> = vec![];

        for project_root in all_project_roots.iter() {
            info!("Project root {:#?}", project_root);
            let project_type = ProjectType::from_repo_dir(&project_root);
            info!("Project type {:#?}", project_type);

            resolve_dependencies(&project_root, &project_type)?;
            compile_project(&project_root, &project_type)?;

            let project_paths_config = match project_type {
                ProjectType::Foundry => Self::parse_foundry_config(&project_root)?,
                ProjectType::Hardhat => Self::parse_foundry_config(&project_root)?,
                _ => {
                    return Err(AppError::UnsupportedProjectType(project_type.clone()));
                }
            };

            let project = Project::builder().paths(project_paths_config).build()?;
            projects.push(project);
        }

        Ok(projects)
    }

    pub fn parse_foundry_config<P>(repo_dir: P) -> Result<ProjectPathsConfig, AppError>
    where
        P: AsRef<Path>,
    {
        let repo_dir = repo_dir.as_ref().to_path_buf();

        let foundry_file = repo_dir.join("foundry.toml");
        if foundry_file.exists() {
            let file_content = fs::read_to_string(foundry_file)
                .map_err(|e| AppError::ParseFoundryConfigError(e.to_string()))?;
            let parsed_toml: FoundryConfig = toml::from_str(&file_content)
                .map_err(|e| AppError::ParseFoundryConfigError(e.to_string()))?;

            let mut project_path_config = ProjectPathsConfig::builder().root(repo_dir.clone());

            if let Some(src_config) = parsed_toml.profile.default.src {
                let src_path = repo_dir.join(src_config);
                project_path_config = project_path_config.sources(src_path);
            }

            if let Some(libs_config) = parsed_toml.profile.default.libs {
                let libs_path: Vec<PathBuf> =
                    libs_config.into_iter().map(|p| repo_dir.join(p)).collect();
                project_path_config = project_path_config.libs(libs_path);
            }

            if let Some(cache_config) = parsed_toml.profile.default.cache_path {
                let cache_file_path = repo_dir
                    .join(cache_config)
                    .join(SOLIDITY_FILES_CACHE_FILENAME);
                project_path_config = project_path_config.cache(cache_file_path);
            }

            if let Some(test_config) = parsed_toml.profile.default.test {
                let test_path = repo_dir.join(test_config);
                project_path_config = project_path_config.tests(test_path);
            }

            let out_config = parsed_toml.profile.default.out.unwrap_or("out".to_owned());
            let out_path = repo_dir.join(out_config);
            project_path_config = project_path_config.artifacts(out_path);

            Ok(project_path_config.build()?)
        } else {
            return Err(AppError::ParseFoundryConfigError(
                "Missing foundry.toml".to_owned(),
            ));
        }
    }

    pub fn parse_hardhat_config<P>(repo_dir: P) -> Result<ProjectPathsConfig, AppError>
    where
        P: AsRef<Path>,
    {
        let repo_dir = repo_dir.as_ref().to_path_buf();

        let mut hardhat_file = repo_dir.join("hardhat.config.ts");
        if !hardhat_file.exists() {
            hardhat_file = repo_dir.join("hardhat.config.js");
            if !hardhat_file.exists() {
                return Err(AppError::ParseHardhatConfigError(
                    "Missing hardhat config".to_owned(),
                ));
            }
        }

        ProjectPathsConfig::hardhat(&repo_dir).map_err(|e| AppError::from(e))
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

/// Find directories that contains config file
pub fn find_all_project_roots<P>(repo_dir: P) -> Vec<PathBuf>
where
    P: AsRef<Path>,
{
    let repo_dir = repo_dir.as_ref().to_path_buf();
    let mut project_root_mapping: HashSet<PathBuf> = HashSet::new();

    let ignore_dirs = vec!["lib", "libs", "node_modules"];
    let valid_config_files = vec![
        "foundry.toml",
        "hardhat.config.js",
        "hardhat.config.ts",
        "truffle-config.js",
    ];

    let mut childs: Vec<PathBuf> = vec![];
    for entry in WalkDir::new(repo_dir.clone())
        .max_depth(2) // we only support 2 levels depth
        .into_iter()
        .filter_entry(|entry| {
            // filter out library directories
            let dir_name = entry.file_name().to_string_lossy().to_string();
            !(entry.file_type().is_dir() && ignore_dirs.contains(&dir_name.as_str()))
        })
        .filter_map(Result::ok)
    {
        if entry.file_type().is_file()
            && valid_config_files
                .contains(&entry.file_name().to_string_lossy().to_string().as_str())
        {
            let child_dir = entry.path().parent().unwrap().to_path_buf();
            if !project_root_mapping.contains(&child_dir) {
                project_root_mapping.insert(child_dir.clone());
                childs.push(child_dir);
            }
        }
    }
    childs
}

#[instrument(fields(repo_dir, project_type))]
pub fn resolve_dependencies<P>(repo_dir: P, project_type: &ProjectType) -> Result<(), AppError>
where
    P: AsRef<Path>,
{
    info!("Resolving project dependencies...");
    let repo_dir = fs::canonicalize(repo_dir)?;
    match project_type {
        ProjectType::Hardhat => {
            let output = Command::new("npm")
                .args(&["install"])
                .current_dir(&repo_dir)
                .output()?;

            if output.status.success() {
                info!("Finish running `npm install`");
            } else {
                let error_message = String::from_utf8_lossy(&output.stderr);
                error!("Resolve dependencies error {:#?}", error_message);
                return Err(AppError::ResolveDependenciesError(
                    error_message.to_string(),
                ));
            }

            // ignore contructions below if we've already got foundry.toml

            let foundry_file = repo_dir.join("foundry.toml");
            if foundry_file.exists() {
                return Ok(());
            }

            let output = Command::new("npm")
                .args(&["install", "--save-dev", "@nomicfoundation/hardhat-foundry"])
                .current_dir(&repo_dir)
                .output()?;

            if output.status.success() {
                info!("Finish installing @nomicfoundation/hardhat-foundry");
            } else {
                let error_message = String::from_utf8_lossy(&output.stderr);
                error!("Resolve dependencies error {:#?}", error_message);
                return Err(AppError::ResolveDependenciesError(
                    error_message.to_string(),
                ));
            }

            // update hardhat file
            let mut hardhat_file = repo_dir.join("hardhat.config.ts");
            if !hardhat_file.exists() {
                hardhat_file = repo_dir.join("hardhat.config.js");
                if !hardhat_file.exists() {
                    return Err(AppError::ParseHardhatConfigError(
                        "Missing hardhat config".to_owned(),
                    ));
                } else {
                    push_first_to_file(
                        hardhat_file,
                        r#"require("@nomicfoundation/hardhat-foundry");"#,
                    )?;
                }
            } else {
                push_first_to_file(
                    hardhat_file,
                    r#"import "@nomicfoundation/hardhat-foundry";"#,
                )?;
            }

            // init foundry

            let output = Command::new("npx")
                .args(&["hardhat", "init-foundry"])
                .current_dir(&repo_dir)
                .output()?;

            if output.status.success() {
                info!("Finish resolve dependencies");
            } else {
                let error_message = String::from_utf8_lossy(&output.stderr);
                error!("Resolve dependencies error {:#?}", error_message);
                return Err(AppError::ResolveDependenciesError(
                    error_message.to_string(),
                ));
            }
            Ok(())
        }
        ProjectType::Foundry => Ok(()),
        _ => Ok(()),
    }
}

#[instrument(fields(repo_dir, project_type))]
pub fn compile_project<P>(repo_dir: P, project_type: &ProjectType) -> Result<(), AppError>
where
    P: AsRef<Path>,
{
    info!("Compiling project...");
    let repo_dir = fs::canonicalize(repo_dir)?;
    match project_type {
        ProjectType::Foundry | ProjectType::Hardhat => {
            let output = Command::new("forge")
                .args(&["build"])
                .current_dir(&repo_dir)
                .output()?;

            if output.status.success() {
                info!("Finish compile project");
                Ok(())
            } else {
                let error_message = String::from_utf8_lossy(&output.stderr);
                error!("Compile project error {:#?}", error_message);
                return Err(AppError::ProjectCompileError(error_message.to_string()));
            }
        }
        // ProjectType::Hardhat => {
        //     let output = Command::new("npx")
        //         .args(&["hardhat", "compile"])
        //         .current_dir(&repo_dir)
        //         .output()?;

        //     if output.status.success() {
        //         info!("Finish compile project");
        //         Ok(())
        //     } else {
        //         let error_message = String::from_utf8_lossy(&output.stderr);
        //         error!("Compile project error {:#?}", error_message);
        //         return Err(AppError::ProjectCompileError(error_message.to_string()));
        //     }
        // }
        _ => {
            return Err(AppError::UnsupportedProjectType(project_type.clone()));
        }
    }
}

fn push_first_to_file<P>(file_path: P, text_to_write: &str) -> Result<(), AppError>
where
    P: AsRef<Path>,
{
    let mut file = File::open(&file_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    // Update the first line with the new text
    let updated_content = {
        let mut lines = content.lines().collect::<Vec<_>>();
        if !lines.is_empty() {
            lines[0] = text_to_write;
        }
        lines.join("\n")
    };

    // Write the updated content back to the file
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&file_path)?;
    file.write_all(updated_content.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_resolve_dependencies() {
        let repo_dir = PathBuf::from("contests/2023-06-lybra");
        let project_type = ProjectType::Hardhat;

        resolve_dependencies(&repo_dir, &project_type).unwrap();
    }

    #[test]
    fn test_compile_project() {
        let repo_dir = PathBuf::from("contests/2023-06-lybra");
        let project_type = ProjectType::Hardhat;

        compile_project(&repo_dir, &project_type).unwrap();
    }

    #[test]
    fn test_project_resolver_parse() {
        let repo_dir = PathBuf::from("contests/2023-06-dodo");
        let project = ProjectResolver::parse(&repo_dir).unwrap();

        println!("{:#?}", project);
    }
}
