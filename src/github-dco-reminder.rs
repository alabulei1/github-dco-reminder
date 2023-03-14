use anyhow::Error;
use github_flows::{get_octo, listen_to_event, EventPayload};
use lazy_static::lazy_static;
use regex::Regex;
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

    // let commits_url = format!("{}/commits", pull_request_url);
    // let uri = Uri::try_from(commits_url.as_str()).unwrap();

    //     let full_url = Url::parse("https://example.com/path/to/resource").unwrap();
    // let base_url = Url::parse("https://api.github.com/").unwrap();
    let path_segments = commits_url.path_segments().unwrap();
    let commits_url_route = path_segments.collect::<Vec<&str>>().join("/");

    send_message_to_channel("ik8", "general", commits_url_route.to_string());

    // let response: Result<octocrab_wasi::from_response, github_flows::octocrab::Error> = octocrab.get(commits_url_route, None::<&()>).await;

    // // .json::<Vec<RepoCommit>>()
    // // .await;

    // if let Ok(res) = response {
    //     send_message_to_channel("ik8", "general", res.to_string());
    // };
    // // let body = response.unwrap().text().await.unwrap();
    // send_message_to_channel("ik8", "general", json_repo_commits.to_string());

    // let commits: Vec<&str> = json
    //     .iter()
    //     .filter_map(|j| j["commit"]["message"].as_str())
    //     .collect();

    // let is_dco_ok = commits
    //     .iter()
    //     .map(|c| {
    //         let msg = c.lines().last().unwrap_or_default();
    //         RE.is_match(msg)
    //     })
    //     .all(std::convert::identity);

    // let msg: &str = if is_dco_ok { "dco ok" } else { "dco wrong" };
    // let body = format!("@{creator}, {msg}");
    // send_message_to_channel("ik8", "general", body.clone());

    // let _ = octocrab
    //     .issues(owner, repo)
    //     .create_comment(pull_number, body)
    //     .await;
}
