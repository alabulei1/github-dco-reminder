use github_flows::{
    get_octo, listen_to_event, octocrab::models::pulls::PullRequestAction,
    octocrab::models::repos::Commit, EventPayload,
};
use slack_flows::send_message_to_channel;
use tokio::*;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() -> anyhow::Result<()> {
    let owner = "jaykchen";
    let repo = "vitesse-lite";

    listen_to_event(owner, repo, vec!["push", "created"], |payload| {
        handler(owner, repo, payload)
    })
    .await;

    Ok(())
}

// push_url = https://github.com/jaykchen/vitesse-lite/commit/930900e8b66d7d97c9bbdc7d57a1260fe1851a96

async fn handler(owner: &str, repo: &str, payload: EventPayload) {
    let octo = get_octo(Some(String::from(owner)));

    match payload {
        EventPayload::PushEvent(e) => {
            let commits = e.commits;

            for commit in commits {
                let user = commit.author.name;
                let commit_sha = commit.sha;
                let commit_message = commit.message;
                let text = format!("sha: {commit_sha}, message: {commit_message}");
                send_message_to_channel("ik8", "general", text);
                let is_signed_off_by = commit_message.contains("Signed-off-by:");

                if !is_signed_off_by {
                    let body = format!("@{user} please DCO sign your commit");

                    send_message_to_channel("ik8", "general", "inside commenting func".to_string());
                    let _ = octo
                        .commits(owner, repo)
                        .create_comment(commit_sha, body)
                        .send()
                        .await;
                }
            }
        }

        _ => (),
    }
}
