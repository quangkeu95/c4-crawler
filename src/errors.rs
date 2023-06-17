use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    SolcIoError(#[from] ethers_solc::error::SolcIoError),
    #[error(transparent)]
    SolcError(#[from] ethers_solc::error::SolcError),
    #[error(transparent)]
    WebDriverSessionError(#[from] fantoccini::error::NewSessionError),
    #[error(transparent)]
    WebDriverCmdError(#[from] fantoccini::error::CmdError),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    UnknownError(#[from] anyhow::Error),
}
