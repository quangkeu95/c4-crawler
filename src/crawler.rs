use crate::errors::AppError;
use crate::types::{Contest, ContestStatus};
use futures::stream::StreamExt;
use lazy_static::lazy_static;
use rr_logging::info;
use std::sync::Arc;
use std::time::Duration;

use headless_chrome::{Browser, Element, Tab};

const ONGOING_CONTESTS_SELECTOR: &'static str =
    "body > div.wrapper__grid > main > div > div > section:nth-child(1) > div > div > div > header > div.contest-tile__details-wrapper > h2 > a";
const UPCOMING_CONTESTS_SELECTOR: &'static str =
    "body > div.wrapper__grid > main > div > div > section:nth-child(2) > div > div > div > header > div.contest-tile__details-wrapper > h2 > a";

const CONTEST_URI_SELECTOR: &'static str = "div ";
const CONTEST_REPO_URI_SELECTOR: &'static str  = "body > div.wrapper__grid > main > div > section > div.contest-page__top-content > div.contest-page__button-wrapper > a";
const CONTEST_NAME_SELECTOR: &'static str  = "body > div.wrapper__grid > main > div > section > div.contest-page__top-content > div.contest-page__project > div:nth-child(2) > h1";
const CONTEST_DESCRIPTION_SELECTOR: &'static str  = "body > div.wrapper__grid > main > div > section > div.contest-page__top-content > div.contest-page__project > div:nth-child(2) > p";
const CONTEST_STATUS_SELECTOR: &'static str =
    "body > div.wrapper__grid > main > div > section > div.contest-page__status-bar > div > span";

pub const C4_URI: &'static str = "https://code4rena.com";
pub const C4_CONTEST_URI: &'static str = "https://code4rena.com/contests";

/// Fetch all ongoing and upcoming contests on C4
pub async fn fetch_all_contests(uri: &str) -> Result<Vec<Contest>, AppError> {
    let mut result: Vec<Contest> = vec![];

    let browser = Browser::default()?;

    let tab = browser.new_tab()?;

    tab.navigate_to(C4_CONTEST_URI)?;

    let mut ongoing_contests = tab.wait_for_elements(ONGOING_CONTESTS_SELECTOR)?;
    let upcoming_contests = tab.wait_for_elements(UPCOMING_CONTESTS_SELECTOR)?;

    info!("Got {:} ongoing contests", ongoing_contests.len());
    info!("Got {:} upcoming contests", upcoming_contests.len());
    ongoing_contests.extend(upcoming_contests);
    let contests = ongoing_contests;

    let contests_uri: Vec<String> = contests
        .iter()
        .filter_map(|element| {
            if let Some(contest_detail_uri) = get_element_single_attribute(element, "href") {
                let contest_detail_uri = format!("{}{}", C4_URI, contest_detail_uri);
                Some(contest_detail_uri)
            } else {
                None
            }
        })
        .collect();

    let mut stream_result = futures::stream::iter(contests_uri)
        .map(|contest_uri| {
            let tab = browser.new_tab().expect("Failed to spawn new tab");
            tokio::spawn(extract_contest(tab, contest_uri))
        })
        .buffer_unordered(10); // allow buffer 10 items

    while let Some(s) = stream_result.next().await {
        // extract tokio join error
        let s = s?;
        // extract result
        let contest = s?;
        info!("Got contest {:?}", contest);
        result.push(contest);
    }

    Ok(result)
}

async fn extract_contest<'a>(
    tab: Arc<Tab>,
    contest_detail_uri: String,
) -> Result<Contest, AppError> {
    tab.navigate_to(&contest_detail_uri)?;

    // get contest name
    let name = tab
        .wait_for_element(CONTEST_NAME_SELECTOR)?
        .get_inner_text()?;

    // get contest description
    let description = tab
        .wait_for_element(CONTEST_DESCRIPTION_SELECTOR)?
        .get_inner_text()?;

    let mut contest_repo_uri: Option<String> = None;
    if let Ok(repo_uri) =
        tab.wait_for_element_with_custom_timeout(CONTEST_REPO_URI_SELECTOR, Duration::from_secs(5))
    {
        contest_repo_uri = get_element_single_attribute(&repo_uri, "href");
    }

    let mut contest_status: ContestStatus = ContestStatus::Upcoming;

    let status = tab
        .wait_for_element_with_custom_timeout(CONTEST_STATUS_SELECTOR, Duration::from_secs(5))?
        .get_inner_text()?;
    if status.contains("Live") {
        contest_status = ContestStatus::Ongoing;
    }

    Ok(Contest {
        name,
        description,
        uri: contest_detail_uri,
        repo_uri: contest_repo_uri,
        status: contest_status,
        contracts: vec![],
    })
}

fn get_element_single_attribute<'a>(element: &'a Element, attribute: &str) -> Option<String> {
    if let Ok(attributes) = element.get_attributes() {
        if let Some(attributes) = attributes {
            if let Some(attr_index) = attributes.iter().position(|v| v.as_str() == attribute) {
                if attr_index < attributes.len() - 1 {
                    return Some(attributes[attr_index + 1].clone());
                }
            }
        }
    }
    return None;
}
