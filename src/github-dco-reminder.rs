use anyhow::Error;
use github_flows::{
    get_octo, listen_to_event,
    octocrab::models::repos::{Commit, RepoCommit},
    octocrab::{FromResponse, Octocrab},
    EventPayload,
};
use http_req::uri::Uri;
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

    if let EventPayload::PullRequestEvent(e) = payload {
        let pull = e.pull_request;
        let mut full_name = "".to_string();

        match pull.repo {
            None => {
                return;
            }

            Some(repo) => {
                full_name = repo.full_name.unwrap_or("no repo name found".to_string());
            }
        };

        let pull_number = pull.number;

        let commits_url =
            format!("https://api.github.com/repos/{full_name}/pulls/{pull_number}/commits");
        // "https://api.github.com/repos/jaykchen/vitesse-lite/pulls/22/commits"
        // let uri = Uri::try_from(commits_url.as_str()).unwrap();

        send_message_to_channel("ik8", "general", commits_url.clone());
        let json = octocrab
            ._get(commits_url, None::<&()>)
            .await
            .unwrap()
            .json::<Vec<RepoCommit>>()
            .await;

        let mut is_dco_ok = false;

        'outer: {
            match json {
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

        // if let Ok(commits) = get_commits(octocrab, commits_url).await {
        //     let is_dco_ok = commits
        //         .iter()
        //         .map(|c| {
        //             let msg = c.lines().last().unwrap_or_default();
        //             RE.is_match(msg)
        //         })
        //         .all(std::convert::identity);

        let creator = &pull.user.unwrap().login;

        let msg: &str = if is_dco_ok { "dco ok" } else { "dco wrong" };
        let body = format!("@{creator}, {msg}");
        send_message_to_channel("ik8", "general", body.clone());

        let _ = octocrab
            .issues(owner, repo)
            .create_comment(pull_number, body)
            .await;
        // }
    }
}

// async fn get_commits(octocrab: &Octocrab, commits_url: String) -> anyhow::Result<Vec<String>> {
//     let json = octocrab
//         ._get(&commits_url, None::<&()>)
//         .await?
//         .json::<Vec<CommitMessage>>()
//         .await?;
//     Ok(json.into_iter().map(|c| c.message).collect())
// }

// async fn get_commits(octocrab: &Octocrab, commits_url: String) -> anyhow::Result<Vec<String>> {
//     let response = octocrab._get(&commits_url, None::<&()>).await?;
//     let body = response.text().await?;
//     let mut comments = Vec::<String>::new();

//     match serde_json::from_str::<serde_json::Value>(&body) {
//         Err(_e) => {
//             return Err(_e.into());
//         }
//         Ok(json) => {
//             for j in json.as_array() {
//                 if let Some(commit) = j["commit"]["message"].as_str() {
//                     comments.push(commit);
//                 }
//             }
//             return Ok(comments);
//         }
//     }
// }
