use serde::Deserialize;

use crate::errors::AppError;

#[derive(Debug, Deserialize)]
struct GithubRepoApiResponse {
    id: usize,
}

pub async fn is_repo_private(repo_uri: &str) -> Result<bool, AppError> {
    let repo_name = repo_uri.replace("https://github.com/", "");
    let repo_api = format!("https://api.github.com/repos/{}", repo_name);

    let client = reqwest::Client::new();
    // let text = client
    //     .get(&repo_api)
    //     .header("User-Agent", "quangkeu95")
    //     .send()
    //     .await?
    //     .text()
    //     .await?;
    // println!("{:?}", text);

    if let Ok(_res) = client
        .get(&repo_api)
        .header("User-Agent", "quangkeu95")
        .send()
        .await?
        .json::<GithubRepoApiResponse>()
        .await
    {
        return Ok(false);
    } else {
        return Ok(true);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use claims::*;

    #[tokio::test]
    async fn test_is_repo_private() {
        let is_private =
            assert_ok!(is_repo_private("https://github.com/sherlock-audit/2023-06-gfx").await);
        assert!(is_private);

        let is_private =
            assert_ok!(is_repo_private("https://github.com/sherlock-audit/2023-06-dodo").await);
        assert!(!is_private);
    }
}
