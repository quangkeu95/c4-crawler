use std::path::{Path, PathBuf};

use ethers::{etherscan::contract::ContractMetadata, types::Bytes};
use ethers_solc::{
    artifacts::{Ast, BytecodeObject, NodeType},
    cache::SolFilesCache,
    Artifact, ArtifactOutput, ConfigurableArtifacts, Project,
};
use rayon::prelude::*;
use rr_logging::info;
use semver::Version;

use crate::{
    errors::AppError,
    types::{Contract, ContractBytecode, ContractFromArtifact, ContractKind},
};

pub struct ContractResolver {}

impl ContractResolver {
    /// Extract bytecode from artifact
    pub fn get_contract_bytecode_from_artifact<AT>(artifact: &AT) -> Option<ContractBytecode>
    where
        AT: Artifact,
    {
        let bytecode_object = artifact.get_bytecode_object()?;
        match bytecode_object.into_owned() {
            BytecodeObject::Bytecode(b) => Some(ContractBytecode::from(b.to_string())),
            BytecodeObject::Unlinked(s) => Some(ContractBytecode::from(s)),
        }
    }

    /// Get imported files from artifact
    pub fn get_imported_files_from_artifact(ast: &Ast) -> Vec<PathBuf> {
        ast.nodes
            .iter()
            .filter_map(|node| {
                if matches!(node.node_type, NodeType::ImportDirective) {
                    if let Some(absolute_path) = node.other["absolutePath"].as_str() {
                        Some(PathBuf::from(absolute_path))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    // /// Get all contract artifacts from project
    // pub fn get_contract_artifacts_from_project<T>(
    //     project: &Project<T>,
    // ) -> Result<Vec<ContractFromArtifact>, AppError>
    // where
    //     T: ArtifactOutput,
    // {
    //     let artifact_dir = project.artifacts_path();
    //     let artifact_files = files_with_extension_from_dir(artifact_dir, "json");
    //     let artifacts = Project::<T>::read_cached_artifacts(&artifact_files)?;

    //     info!("Number of artifacts {:?}", artifacts.len());

    //     let contracts = artifacts
    //         .par_iter()
    //         .filter_map(|(artifact_path, artifact)| {
    //             let name = Project::<T>::contract_name(artifact_path)?;
    //             let bytecode = Self::get_contract_bytecode_from_artifact(artifact)?;
    //             let kind = ContractKind::from(bytecode);
    //             let artifact_path = artifact_path.clone();

    //             Some(ContractFromArtifact {
    //                 name,
    //                 kind,
    //                 artifact_path,
    //             })
    //         })
    //         .collect::<Vec<ContractFromArtifact>>();

    //     info!("Number of parsed artifacts {:?}", contracts.len());
    //     Ok(contracts)
    // }

    /// Get contract artifact from file
    pub fn get_contract_artifact_from_file<T>(
        artifact_path: PathBuf,
    ) -> Option<ContractFromArtifact>
    where
        T: ArtifactOutput,
    {
        let artifact = Project::<T>::read_cached_artifact(&artifact_path).ok()?;
        let name = Project::<T>::contract_name(artifact_path.clone())?;
        let bytecode = Self::get_contract_bytecode_from_artifact(&artifact)?;
        let kind = ContractKind::from(bytecode);

        Some(ContractFromArtifact {
            name,
            kind,
            artifact_path,
        })
    }

    /// Get all contracts from project
    pub fn get_contracts_from_project<T>(project: &Project<T>) -> Result<Vec<Contract>, AppError>
    where
        T: ArtifactOutput,
    {
        let solc_cache = project.read_cache_file()?;
        let project_root = project.root();
        let source_files: Vec<PathBuf> = solc_cache
            .files
            .iter()
            .map(|(file, _)| file.clone())
            .collect();
        info!("Number of Solidity files = {:?}", source_files.len());

        let mut contracts = source_files
            .par_iter()
            .filter_map(|file| {
                // info!("Handling file {:?}", file);
                let contracts = Self::get_contracts_from_cache(
                    solc_cache.clone(),
                    project_root.clone(),
                    file.clone(),
                );

                if contracts.len() > 0 {
                    Some(contracts)
                } else {
                    None
                }
            })
            .flatten()
            .collect::<Vec<Contract>>();
        contracts.sort_by(|a, b| a.partial_cmp(b).unwrap());

        info!("Number of contracts = {:?}", contracts.len());

        Ok(contracts)
    }

    pub fn get_contracts_from_cache(
        solc_cache: SolFilesCache,
        project_root: PathBuf,
        cache_entry_path: PathBuf,
    ) -> Vec<Contract> {
        let cache_entry = solc_cache.files.get(&cache_entry_path);
        if let Some(cache_entry) = cache_entry {
            let mut contracts: Vec<Contract> = vec![];

            for (version, artifact_file) in cache_entry.artifacts_versions() {
                let name = artifact_file
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();

                let artifact =
                    Project::<ConfigurableArtifacts>::read_cached_artifact(artifact_file).ok();
                if artifact.is_none() {
                    continue;
                }
                let artifact = artifact.unwrap();
                if artifact.ast.is_none() {
                    continue;
                }
                let ast = artifact.clone().ast.unwrap();

                let imported_files = Self::get_imported_files_from_artifact(&ast);
                let imported_artifacts_path: Vec<PathBuf> = imported_files
                    .into_iter()
                    .filter_map(|file| {
                        let file = project_root.join(file);
                        let entry = solc_cache.files.get(&file)?;
                        let artifacts: Vec<PathBuf> =
                            entry.artifacts().map(|item| item.to_owned()).collect();
                        Some(artifacts)
                    })
                    .flatten()
                    .collect();

                let mut imported_contracts: Vec<ContractFromArtifact> = imported_artifacts_path
                    .iter()
                    .filter_map(|imported_file| {
                        Self::get_contract_artifact_from_file::<ConfigurableArtifacts>(
                            imported_file.clone(),
                        )
                    })
                    .collect();

                // sort interface first

                imported_contracts.sort_by(|a, b| a.partial_cmp(b).unwrap());

                let bytecode = Self::get_contract_bytecode_from_artifact(&artifact);
                if bytecode.is_none() {
                    continue;
                }
                let bytecode = bytecode.unwrap();
                let kind = ContractKind::from(bytecode);

                let c = Contract {
                    name,
                    kind,
                    version: version.clone(),
                    // artifact_file: artifact_file.clone(),
                    imported_contracts,
                };
                contracts.push(c);
            }

            contracts
        } else {
            vec![]
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContractCacheEntry {
    pub name: String,
    pub version: Version,
    pub artifact_file: PathBuf,
    pub imported_files: Vec<PathBuf>,
}

#[cfg(test)]
mod test {

    use ethers_solc::{Project, ProjectPathsConfig};

    use super::*;

    #[test]
    fn test_ContractResolver_get_contracts_from_cache() {
        let project_path = "tests/2023-05-maia";
        let project = Project::builder()
            .paths(
                ProjectPathsConfig::builder()
                    .root(project_path)
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap();

        let solc_cache = project.read_cache_file().unwrap();
        let root_dir = project.root();
        let cache_entry_path = root_dir.join("src/talos/factories/TalosStrategyStakedFactory.sol");
        let result = ContractResolver::get_contracts_from_cache(
            solc_cache,
            root_dir.clone(),
            cache_entry_path,
        );
        println!("{:#?}", result);
    }

    // #[ignore]
    // #[test]
    // fn test_get_contract_artifacts_from_project() {
    //     let project_path = "tests/2023-05-maia";
    //     let project = Project::builder()
    //         .paths(
    //             ProjectPathsConfig::builder()
    //                 .root(project_path)
    //                 .build()
    //                 .unwrap(),
    //         )
    //         .build()
    //         .unwrap();
    //     ContractResolver::get_contract_artifacts_from_project(&project).unwrap();
    // }

    #[test]
    fn test_get_contracts_from_project() {
        let project_path = "tests/2023-05-maia";
        let project = Project::builder()
            .paths(
                ProjectPathsConfig::builder()
                    .root(project_path)
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap();
        let contracts = ContractResolver::get_contracts_from_project(&project).unwrap();
        for contract in contracts {
            println!("{:#?}", contract);
        }
    }
}
