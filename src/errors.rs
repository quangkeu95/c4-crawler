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
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    UnknownError(#[from] anyhow::Error),
}
