use anyhow::Error;
use github_flows::octocrab::Result as OctoResult;
use github_flows::{get_octo, listen_to_event, EventPayload};
use lazy_static::lazy_static;
use regex::Regex;
// use reqwest::Response;
use dotenv::dotenv;
use http_req::{
    request::{Method, Request},
    uri::Uri,
};
use std::env;
// use octocrab_wasi::{FromResponse};
// use serde_json::Value;
use slack_flows::send_message_to_channel;
use tokio::*;
#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() -> anyhow::Result<()> {
    let owner = "jaykchen";
    let repo = "vitesse-lite";

    listen_to_event(
        owner,
        repo,
        vec![
            "pull_request",
            "pull_request_comment",
            "pull_request_comment_review",
        ],
        |payload| handler(owner, repo, payload),
    )
    .await;

    Ok(())
}

async fn handler(owner: &str, repo: &str, payload: EventPayload) {
    dotenv().ok();
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r#"Signed-off-by: \w+ <[\w._%+-]+@[\w.-]+\.\w{2,4}>"#).unwrap();
    }

    let octocrab = get_octo(Some(String::from(owner)));
    let mut pull = None;

    match payload {
        EventPayload::PullRequestEvent(e) => {
            pull = Some(e.pull_request);
        }

        EventPayload::PullRequestReviewEvent(e) => {
            pull = Some(e.pull_request);
        }
        EventPayload::PullRequestReviewCommentEvent(e) => {
            pull = Some(e.pull_request);
        }

        _ => (),
    };

    let (commits_url, pull_number, creator) = match pull {
        Some(p) => (
            p.commits_url.unwrap().to_string(),
            p.number,
            p.user.unwrap().login,
        ),
        None => return,
    };

    // let path_segments = commits_url.path_segments().unwrap();
    // let commits_url_route = path_segments.collect::<Vec<&str>>().join("/");

    let uri = Uri::try_from(commits_url.as_str()).unwrap();
    send_message_to_channel("ik8", "general", commits_url.to_string());

    let TOKEN = env::var("GITHUB_TOKEN").unwrap();
    let mut writer = Vec::new();
    _ = Request::new(&uri)
        .method(Method::GET)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "Github Connector of Second State Reactor")
        .header("Authorization", &format!("Bearer {}", TOKEN))
        .send(&mut writer)
        .map_err(|e| {})
        .unwrap();

    let text = String::from_utf8_lossy(&writer);
    send_message_to_channel("ik8", "general", text.to_string());

    let json: Vec<serde_json::Value> = serde_json::from_str(&text).map_err(|e| {}).unwrap();

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


    send_message_to_channel("ik8", "general", creator.to_string());

    // let client = octocrab._client();

    // client.new().get(&commits_url_route).send().await.unwrap();

    // let response: OctoResult<Response> = octocrab._get(commits_url, None::<&()>).await;

    // let body = response.unwrap().text().await.unwrap();
    // send_message_to_channel("ik8", "general", body.to_string());
}
