use crate::errors::AppError;
use crate::types::{Contest, ContestStatus};
use fantoccini::{elements::Element, Client, ClientBuilder, Locator};
use lazy_static::lazy_static;
use paris::info;
use std::time::Duration;

const ONGOING_CONTESTS_SELECTOR: Locator =
    Locator::Css("body > div.wrapper__grid > main > div > div > section:nth-child(1) > div > div");
const UPCOMING_CONTESTS_SELECTOR: Locator =
    Locator::Css("body > div.wrapper__grid > main > div > div > section:nth-child(2) > div > div");

const CONTEST_URI_SELECTOR: Locator =
    Locator::Css("div > header > div.contest-tile__details-wrapper > h2 > a");
const CONTEST_REPO_URI_SELECTOR: Locator = Locator::Css("body > div.wrapper__grid > main > div > section > div.contest-page__top-content > div.contest-page__button-wrapper > a");
const CONTEST_NAME_SELECTOR: Locator = Locator::Css("body > div.wrapper__grid > main > div > section > div.contest-page__top-content > div.contest-page__project > div:nth-child(2) > h1");
const CONTEST_DESCRIPTION_SELECTOR: Locator = Locator::Css("body > div.wrapper__grid > main > div > section > div.contest-page__top-content > div.contest-page__project > div:nth-child(2) > p");
const CONTEST_STATUS_SELECTOR: Locator = Locator::Css(
    "body > div.wrapper__grid > main > div > section > div.contest-page__status-bar > div > span",
);

pub const C4_URI: &'static str = "https://code4rena.com";
pub const C4_CONTEST_URI: &'static str = "https://code4rena.com/contests";

/// REMEMBER to close client connection after used
pub async fn new_webdriver_client(
    webdriver_url: &str,
) -> Result<Client, fantoccini::error::NewSessionError> {
    ClientBuilder::native().connect(webdriver_url).await
}

/// Fetch all ongoing and upcoming contests on C4
pub async fn fetch_all_contests(uri: &str, webdriver_url: &str) -> Result<Vec<Contest>, AppError> {
    let mut result: Vec<Contest> = vec![];

    let client = new_webdriver_client(webdriver_url).await?;

    client.goto(uri).await?;

    client.wait().for_element(ONGOING_CONTESTS_SELECTOR).await?;
    let ongoing_contests = client.find_all(ONGOING_CONTESTS_SELECTOR).await?;
    info!("Got {:} ongoing contests", ongoing_contests.len());

    for item in ongoing_contests {
        let contest = extract_contest(&client, &item).await?;
        info!("Got ongoing contest: {:?}", contest);
        result.push(contest);
    }

    client
        .wait()
        .for_element(UPCOMING_CONTESTS_SELECTOR)
        .await?;
    let upcoming_contests = client.find_all(UPCOMING_CONTESTS_SELECTOR).await?;
    info!("Got {:} upcoming contests", upcoming_contests.len());

    for item in upcoming_contests {
        let contest = extract_contest(&client, &item).await?;
        info!("Got upcoming contest: {:?}", contest);
        result.push(contest);
    }

    client.close().await?;
    Ok(result)
}

async fn extract_contest(client: &Client, element: &Element) -> Result<Contest, AppError> {
    let contest_detail_uri = element
        .find(CONTEST_URI_SELECTOR)
        .await?
        .attr("href")
        .await?
        .expect("No contest detail URI");
    let contest_detail_uri = format!("{}{}", C4_URI, contest_detail_uri);

    let current_window = client.window().await?;
    // open new window
    let new_window = client.new_window(true).await?;
    client.switch_to_window(new_window.handle).await?;

    client.goto(&contest_detail_uri).await?;

    // get contest name
    let name = client
        .wait()
        .for_element(CONTEST_NAME_SELECTOR)
        .await?
        .text()
        .await?;

    // get contest description
    let description = client
        .wait()
        .for_element(CONTEST_DESCRIPTION_SELECTOR)
        .await?
        .text()
        .await?;
    let mut contest_repo_uri: Option<String> = None;
    if let Ok(repo_uri) = client
        .wait()
        .at_most(Duration::from_secs(5))
        .for_element(CONTEST_REPO_URI_SELECTOR)
        .await
    {
        contest_repo_uri = repo_uri.attr("href").await?;
    }

    let mut contest_status: ContestStatus = ContestStatus::Upcoming;

    let status = client.find(CONTEST_STATUS_SELECTOR).await?.text().await?;
    if status.contains("Live") {
        contest_status = ContestStatus::Ongoing;
    }

    client.close_window().await?;
    client.switch_to_window(current_window).await?;

    Ok(Contest {
        name,
        description,
        uri: contest_detail_uri,
        repo_uri: contest_repo_uri,
        status: contest_status,
        contracts: vec![],
    })
}
