use dotenv::dotenv;
use github_flows::octocrab::models::repos::RepoCommit;
use github_flows::{get_octo, listen_to_event, EventPayload};
use http_req::{
    request::{Method, Request},
    uri::Uri,
};
use lazy_static::lazy_static;
use regex::Regex;
use std::env;
use tokio::*;
#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() -> anyhow::Result<()> {
    let owner = "WasmEdge";
    let repo = "WasmEdge";

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

    let uri = Uri::try_from(commits_url.as_str()).unwrap();

    let token = env::var("GITHUB_TOKEN").unwrap();

    let mut writer = Vec::new();
    _ = Request::new(&uri)
        .method(Method::GET)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "Github Connector of Second State Reactor")
        .header("Authorization", &format!("Bearer {}", token))
        .send(&mut writer)
        .map_err(|_e| {})
        .unwrap();

    let text = String::from_utf8_lossy(&writer);

    let repo_commit_array: Vec<RepoCommit> = serde_json::from_str(&text).map_err(|_e| {}).unwrap();

    let is_dco_ok = repo_commit_array
        .iter()
        .map(|j| {
            let msg = j.commit.message.lines().last().unwrap_or_default();
            RE.is_match(msg)
        })
        .all(std::convert::identity);

    let msg: &str = if is_dco_ok { "Thanks for your contribution! The miantainers will review your PR soon." } else { "Thanks for your contributin. It seems that your DCO test is failed. Please fix it.'" };
    let body = format!("@{creator}, {msg}");
    let _ = octocrab
        .issues(owner, repo)
        .create_comment(pull_number, body)
        .await;
}
