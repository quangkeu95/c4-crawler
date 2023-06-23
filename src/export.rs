use std::path::PathBuf;

use crate::types::Contract;

pub fn export_contracts_to_file<P>(repo_name: &str, contracts: Vec<Contract>, output_filepath: P)
where
    P: AsRef<PathBuf>,
{
}
