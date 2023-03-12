use anyhow::Error;
use github_flows::{get_octo, listen_to_event, octocrab::models::repos::RepoCommit, EventPayload};
use lazy_static::lazy_static;
use regex::Regex;
// use serde_json::Value;
use slack_flows::send_message_to_channel;
use tokio::*;
use url::Url;
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

    let mut pull_request_url = "".to_string();
    let mut pull_number = 0u64;
    match payload {
        EventPayload::PullRequestEvent(e) => {
            pull_request_url = e.pull_request.url;
            pull_number = e.pull_request.number;
        }

        EventPayload::PullRequestReviewEvent(e) => {
            pull_request_url = e.pull_request.url;
            pull_number = e.pull_request.number;
        }
        EventPayload::PullRequestReviewCommentEvent(e) => {
            pull_request_url = e.pull_request.url;
            pull_number = e.pull_request.number;
        }
        EventPayload::UnknownEvent(e) => {
            let text = e.to_string();

            let val: serde_json::Value = serde_json::from_str(&text).unwrap();
            pull_request_url = val["pull_request"]["url"]
                .as_str()
                .unwrap_or("no url found")
                .to_string();

            pull_number = val["pull_request"]["number"].as_u64().unwrap_or(0);
        }

        _ => (),
    };

    // let url = Url::parse(&pull_request_url).unwrap();
    // let pull_number = url.path_segments().unwrap().last().unwrap();
    let commits_url = format!("{}/commits", pull_request_url);
    // let commits_url =
    //     format!("https://api.github.com/repos/{full_name}/pulls/{pull_number}/commits");

    let json_repo_commits = octocrab
        ._get(commits_url, None::<&()>)
        .await
        .expect("octocrab failed to get data");
        // .json::<Vec<RepoCommit>>()
        // .await;

let body = json_repo_commits.text().await.unwrap();
send_message_to_channel("ik8", "general", body.to_string());

    // let mut is_dco_ok = false;
    // let mut creator = "".to_string();
    // 'outer: {
    //     match json_repo_commits {
    //         Err(_) => {
    //             send_message_to_channel("ik8", "general", "failed to parse RepoCommit".to_string());
    //         }
    //         Ok(repo_commits) => {
    //            send_message_to_channel("ik8", "general", repo_commits[0].clone().url.to_owned());
     
    //             for repo_commit in repo_commits {
    //                 creator = repo_commit.author.unwrap().login;
    //                 let msg = repo_commit.commit.message;
    //                 if !RE.is_match(&msg) {
    //                     break 'outer;
    //                 }
    //             }
    //             is_dco_ok = true;
    //         }
    //     };
    // };

    // let msg: &str = if is_dco_ok { "dco ok" } else { "dco wrong" };
    // let body = format!("@{creator}, {msg}");
    // send_message_to_channel("ik8", "general", body.clone());

    // let _ = octocrab
    //     .issues(owner, repo)
    //     .create_comment(pull_number, body)
    //     .await;
    // }
}
