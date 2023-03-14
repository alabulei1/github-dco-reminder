use anyhow::Error;
use github_flows::octocrab::{Octocrab, Result as OctoResult};
use github_flows::{get_octo, listen_to_event, EventPayload};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Response;
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
        Some(p) => (p.commits_url.unwrap(), p.number, p.user.unwrap().login),
        None => return,
    };

    let path_segments = commits_url.path_segments().unwrap();
    let commits_url_route = path_segments.collect::<Vec<&str>>().join("/");

    send_message_to_channel("ik8", "general", commits_url_route.to_string());

    let response: OctoResult<Response> = octocrab._get(commits_url, None::<&()>).await;

let body = response.unwrap().text().await.unwrap();
    send_message_to_channel("ik8", "general", body.to_string());

  
}
