use anyhow::Error;
use github_flows::{
    get_octo, listen_to_event,
    octocrab::models::repos::{Commit, RepoCommit},
    EventPayload,
};
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;
use slack_flows::send_message_to_channel;
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
    let mut full_name = "".to_string();
    let mut pull_number = 0u64;
    let mut creator = "".to_string();
    match payload {
        EventPayload::PullRequestEvent(e) => {
            let pull = e.pull_request;

            if let Some(repo) = pull.repo {
                full_name = repo.full_name.unwrap_or("no repo name found".to_string());
            }
            pull_number = pull.number;
            creator = pull.user.unwrap().login;

        }

        EventPayload::PullRequestReviewEvent(e) => {
            let pull = e.pull_request;

            if let Some(repo) = pull.repo {
                full_name = repo.full_name.unwrap_or("no full_name found".to_string());
            }
            pull_number = pull.number;
            creator = pull.user.unwrap().login;
        }
        EventPayload::PullRequestReviewCommentEvent(e) => {
            let pull = e.pull_request;

            if let Some(repo) = pull.repo {
                full_name = repo.full_name.unwrap_or("no full_name found".to_string());
            }
            pull_number = pull.number;
            creator = pull.user.unwrap().login;
        }
        EventPayload::UnknownEvent(e) => {
            let text = e.to_string();

            let val: serde_json::Value = serde_json::from_str(&text).unwrap();
            full_name = val["pull_request"]["repo"]["full_name"]
                .as_str()
                .unwrap_or("no full_name found")
                .to_string();

            pull_number = val["pull_request"]["number"].as_u64().unwrap_or(0);
            creator = val["pull_request"]["repo"]["user"]["login"]
                .as_str()
                .unwrap_or("no creator found")
                .to_string();
        }

        _ => (),
    };
    let commits_url =
        format!("https://api.github.com/repos/{full_name}/pulls/{pull_number}/commits");
    // "https://api.github.com/repos/jaykchen/vitesse-lite/pulls/22/commits"
    // let uri = Uri::try_from(commits_url.as_str()).unwrap();

    send_message_to_channel("ik8", "general", commits_url.clone());
    let json_repo_commits = octocrab
        ._get(commits_url, None::<&()>)
        .await
        .expect("octocrab failed to get data")
        .json::<Vec<RepoCommit>>()
        .await;
  
    let mut is_dco_ok = false;

    'outer: {
        match json_repo_commits {
            Err(_) => {
                send_message_to_channel("ik8", "general", "failed to parse RepoCommit".to_string());
            }
            Ok(repo_commits) => {
                for repo_commit in repo_commits {
                    let msg = repo_commit.commit.message;
                    if !RE.is_match(&msg) {
                        break 'outer;
                    }
                }
                is_dco_ok = true;
            }
        };
    };

    let msg: &str = if is_dco_ok { "dco ok" } else { "dco wrong" };
    let body = format!("@{creator}, {msg}");
    send_message_to_channel("ik8", "general", body.clone());

    let _ = octocrab
        .issues(owner, repo)
        .create_comment(pull_number, body)
        .await;
    // }
}
