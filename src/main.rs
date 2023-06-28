use c4_crawler::compiler::{
    clone_or_pull_repo, find_all_contracts, project_dir_from_uri, ProjectType,
};
use c4_crawler::crawler::fetch_all_contests;
use c4_crawler::types::{Contest, Contract};
use rr_logging::{error, info, init_tracing};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // init logging
    std::env::set_var("RUST_LOG", "info,headless_chrome=error");
    init_tracing(None);

    info!("Starting...");

    let all_contests = fetch_all_contests().await?;

    let contests_has_repo: Vec<Contest> = all_contests
        .into_iter()
        .filter(|item| item.repo_uri.is_some())
        .collect();

    for mut contest in contests_has_repo {
        info!("Contest {:#?}", contest);
        let repo_uri = contest.repo_uri.unwrap();
        let repo_dir = project_dir_from_uri(&repo_uri);
        info!("Repo directory {:#?}", repo_dir);

        let project_type = ProjectType::from_repo_dir(&repo_dir);
        info!("Project type {:?}", project_type);

        if let Err(_e) = clone_or_pull_repo(&repo_uri).map_err(|e| {
            error!("Clone repo error: {e:?}");
            e
        }) {
            continue;
        };

        let all_contracts = find_all_contracts(repo_dir).map_err(|e| {
            error!("Find all contracts error {e:?}");
            e
        })?;

        info!("Repo name {:?}", repo_uri);
        for contract in all_contracts.iter() {
            info!("Contract {:#?}", contract);
        }

        contest.contracts = all_contracts;
    }

    // let repo_uri = "https://github.com/code-423n4/2023-05-maia";

    Ok(())
}
