use github_flows::{get_octo, listen_to_event, EventPayload};
use http_req::uri::Uri;
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;
use tokio::*;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() -> anyhow::Result<()> {
    let owner = "jaykchen";
    let repo = "vitesse-lite";

    listen_to_event(owner, repo, vec!["pull_request"], |payload| {
        handler(owner, repo, payload)
    })
    .await;

    Ok(())
}

async fn handler(owner: &str, repo: &str, payload: EventPayload) {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r#"Signed-off-by: \w+ <[\w._%+-]+@[\w.-]+\.\w{2,4}>"#).unwrap();
    }

    let octocrab = get_octo(Some(String::from(owner)));

    if let EventPayload::PullRequestEvent(e) = payload {
        let pull = e.pull_request;
        let full_name = pull
            .repo
            .unwrap()
            .full_name
            .unwrap_or("no repo name found".to_string());

        let pull_number = pull.number;

        let commits_url =
            format!("https://api.github.com/repos/{full_name}/pulls/{pull_number}/commits");
        // "https://api.github.com/repos/jaykchen/vitesse-lite/pulls/22/commits"
        // let uri = Uri::try_from(commits_url.as_str()).unwrap();

        let mut json: Vec<serde_json::Value> = Vec::new();

        match octocrab._get(&commits_url, None::<&()>).await {
            Err(_) => (),

            Ok(response) => match response.text().await {
                Err(_) => (),

                Ok(body) => {
                    // let text = String::from_utf8_lossy(body.as_bytes());
                    json = serde_json::from_slice(body.as_bytes())
                        .map_err(|_e| {})
                        .unwrap();
                }
            },
        };

        let commits: Vec<&str> = json
            .iter()
            .filter_map(|j| j["commit"]["message"].as_str())
            .collect();

        let is_dco_ok = commits
            .iter()
            .map(|c| {
                let msg = c.lines().last().unwrap_or_default();
                RE.is_match(msg)
            })
            .all(std::convert::identity);

        let creator = &pull.user.unwrap().login;

        let msg: &str = if is_dco_ok { "dco ok" } else { "dco wrong" };
        let body = format!("@{creator}, {msg}");
        let _ = octocrab
            .issues(owner, repo)
            .create_comment(pull_number, body)
            .await;
    }
}
