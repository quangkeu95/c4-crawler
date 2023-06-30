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

    for mut contest in all_contests {
        info!("Contest {:#?}", contest);
        let repo_uri = contest.repo_uri.unwrap();
        let repo_dir = project_dir_from_uri(&repo_uri);
        info!("Repo directory {:#?}", repo_dir);

        if let Err(e) = clone_or_pull_repo(&repo_uri) {
            error!("Clone repo error: {e:?}");
            continue;
        };

        let all_contracts = match find_all_contracts(repo_dir) {
            Ok(result) => result,
            Err(e) => {
                error!("Find all contracts error {e:?}");
                continue;
            }
        };

        info!("Found {:#?} contracts", all_contracts.len());

        // info!("Repo name {:?}", repo_uri);
        // for contract in all_contracts.iter() {
        //     info!("Contract {:#?}", contract);
        // }

        contest.contracts = all_contracts;
    }

    // let repo_uri = "https://github.com/code-423n4/2023-05-maia";

    Ok(())
}
