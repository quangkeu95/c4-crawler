use super::Contract;
use derive_more::{From, Into};
use std::path::Path;

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

#[derive(Debug, From, Into)]
pub struct RepoUri(String);

impl RepoUri {
    pub fn to_dir_name(&self) -> Option<&str> {
        Path::new(&self.0).file_name().unwrap().to_str()
    }
}
