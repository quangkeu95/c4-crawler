use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    TokioJoinHandleError(#[from] tokio::task::JoinError),
    #[error(transparent)]
    SolcIoError(#[from] ethers_solc::error::SolcIoError),
    #[error(transparent)]
    SolcError(#[from] ethers_solc::error::SolcError),
    #[error(transparent)]
    ContractError(#[from] ContractError),
    #[error(transparent)]
    DeriveBuilderUninitializedFieldError(#[from] derive_builder::UninitializedFieldError),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    UnknownError(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("Invalid bytecode: {0}")]
    InvalidBytecode(String),
    #[error("Contract not found {0}")]
    ContractNotFound(PathBuf),
    #[error("Contract name not found from artifact file {0}")]
    ContractNameNotFound(PathBuf),
    #[error("Contract builder error: {0}")]
    ContractBuilderError(String),
}
