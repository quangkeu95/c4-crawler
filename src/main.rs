use c4_crawler::compiler::{clone_or_pull_repo, compile, find_all_contracts};
use c4_crawler::crawler::{fetch_all_contests, C4_CONTEST_URI};
use c4_crawler::types::{Contest, Contract};
use paris::{error, info};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    info!("Starting...");

    let all_contests = fetch_all_contests(C4_CONTEST_URI).await?;

    let contests_has_repo: Vec<Contest> = all_contests
        .into_iter()
        .filter(|item| item.repo_uri.is_some())
        .collect();

    let mut result: HashMap<String, Vec<Contract>> = HashMap::new();
    for contest in contests_has_repo {
        let repo_uri = contest.repo_uri.unwrap();
        let repo_dir = clone_or_pull_repo(&repo_uri).map_err(|e| {
            error!("Clone repo error: {e:?}");
            e
        })?;

        let all_contracts = find_all_contracts(repo_dir).map_err(|e| {
            error!("Find all contracts error {e:?}");
            e
        })?;

        info!("Repo name {:?}", repo_uri);
        for contract in all_contracts.iter() {
            info!(
                "Contract name {:?} with bytecode {}",
                contract.name, contract.bytecode
            );
        }

        result.insert(repo_uri, all_contracts);
    }

    // let repo_uri = "https://github.com/code-423n4/2023-05-maia";

    Ok(())
}
