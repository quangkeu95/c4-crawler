use rr_logging::{info, instrument, tracing};
use serde::Deserialize;

use crate::{
    errors::AppError,
    types::{Contest, ContestStatus},
};

use super::ContestCrawler;

const SHERLOCK_CONTESTS_URI: &'static str = "https://app.sherlock.xyz/audits/contests";
const SHERLOCK_CONTESTS_API: &'static str = "https://mainnet-contest.sherlock.xyz/contests";

#[derive(Debug, Deserialize)]
struct SherlockContestApiResponse {
    id: usize,
    status: Status,
    template_repo_name: String,
    title: String,
    short_description: String,
    private: bool,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize)]
enum Status {
    FINISHED,
    RUNNING,
    CREATED,
    SHERLOCK_JUDGING,
    JUDGING,
    ESCALATING,
}

impl Status {
    fn is_ongoing_or_upcoming(&self) -> bool {
        matches!(self, Self::RUNNING) || matches!(self, Self::CREATED)
    }
}

#[derive(Debug, Default)]
pub struct SherlockCrawler {}

impl SherlockCrawler {
    fn contest_uri(contest_id: usize) -> String {
        format!("{}/{contest_id:}", SHERLOCK_CONTESTS_URI)
    }

    fn repo_uri(template_repo_name: &str) -> String {
        format!("https://github.com/{template_repo_name:}")
    }

    fn contest_status(status: &Status) -> ContestStatus {
        match status {
            Status::CREATED => ContestStatus::Upcoming,
            Status::RUNNING => ContestStatus::Ongoing,
            _ => unimplemented!("Unexpected contest status"),
        }
    }
}

#[async_trait::async_trait]
impl ContestCrawler for SherlockCrawler {
    #[instrument(skip_all)]
    async fn fetch_all_contests(&self) -> Result<Vec<Contest>, AppError> {
        let response: Vec<SherlockContestApiResponse> =
            reqwest::get(SHERLOCK_CONTESTS_API).await?.json().await?;

        let result = response
            .iter()
            .filter_map(|item| {
                if item.private || !item.status.is_ongoing_or_upcoming() {
                    return None;
                }

                Some(Contest {
                    name: item.title.to_owned(),
                    description: item.short_description.to_owned(),
                    uri: SherlockCrawler::contest_uri(item.id),
                    repo_uri: Some(SherlockCrawler::repo_uri(&item.template_repo_name)),
                    status: SherlockCrawler::contest_status(&item.status),
                    contracts: vec![],
                })
            })
            .collect::<Vec<Contest>>();

        let ongoing_contests_count = result
            .iter()
            .filter(|item| matches!(item.status, ContestStatus::Ongoing))
            .count();
        let upcoming_contests_count = result
            .iter()
            .filter(|item| matches!(item.status, ContestStatus::Upcoming))
            .count();
        info!("Got {:#?} Sherlock contests. Ongoing contests = {ongoing_contests_count:#?}. Upcoming contests = {upcoming_contests_count:#?}", result.len());

        Ok(result)
    }
}
