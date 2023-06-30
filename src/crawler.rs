use crate::errors::AppError;
use crate::types::{Contest, ContestStatus};
use futures::stream::StreamExt;
use headless_chrome::{Browser, Element, Tab};
use lazy_static::lazy_static;
use rr_logging::{error, info, instrument, tracing, warn};
use std::sync::Arc;
use std::time::Duration;

use self::codearena::CodearenaCrawler;
use self::sherlock::SherlockCrawler;
use self::utils::is_repo_private;

pub mod codearena;
pub mod sherlock;
pub mod utils;

#[async_trait::async_trait]
pub trait ContestCrawler {
    async fn fetch_all_contests(&self) -> Result<Vec<Contest>, AppError>;
}

pub fn get_crawlers() -> Vec<Arc<dyn ContestCrawler>> {
    let mut crawlers: Vec<Arc<dyn ContestCrawler>> = vec![];
    // C4
    let codearena_crawler = CodearenaCrawler::default();
    crawlers.push(Arc::new(codearena_crawler));

    // sherlock
    let sherlock_crawler = SherlockCrawler::default();
    crawlers.push(Arc::new(sherlock_crawler));

    crawlers
}

/// Fetch all contests from Code4rena, Sherlock, Immunefi, Blackhat
#[instrument]
pub async fn fetch_all_contests() -> Result<Vec<Contest>, AppError> {
    let crawlers = get_crawlers();

    let tasks = crawlers.iter().map(|crawler| crawler.fetch_all_contests());

    let mut stream = futures::stream::iter(tasks).buffered(5);

    let mut result = vec![];
    while let Some(response) = stream.next().await {
        if let Err(e) = response {
            error!("Error fetching contests {:#?}", e);
            continue;
        }

        // filter out private contests
        let contests = response.unwrap();
        for contest in contests {
            if contest.repo_uri.is_none() {
                warn!("Skipping private contest {:#?}", contest.name);
                continue;
            }

            let repo_uri = contest.repo_uri.as_ref().unwrap();
            if is_repo_private(&repo_uri).await.unwrap_or(true) {
                warn!("Skipping private contest {:#?}", contest.name);
                continue;
            }
            result.push(contest);
        }
    }
    Ok(result)
}
