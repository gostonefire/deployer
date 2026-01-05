use std::sync::Arc;
use tokio::process::Command;
use anyhow::Result;
use log::{error, info};
use thiserror::Error;
use tokio::sync::RwLock;
use crate::manager_mail::Mail;

/// Runs the deployment script and sends an email notification.
///
/// # Arguments
///
/// * 'script_path' - Path to the master deployment shell script
/// * 'dev_dir' - Development directory in the server
/// * 'scripts_dir' - Directory where scripts are stored
/// * 'repo' - Full Git repository name
/// * 'tag' - Git tag to deploy
/// * 'mail' - Mailer instance for sending notifications
pub async fn run_deploy(script_path: String, dev_dir: String, scripts_dir: String, repo: String, tag: String, mail: Arc<RwLock<Mail>>) {
    let repo_name = repo.rsplit_once('/').map(|(_, name)| name).unwrap_or(&repo);

    let (subject, body) = match deploy(&script_path, &dev_dir, &scripts_dir, repo_name, &tag).await {
        Ok(result) =>
            (
                "Deploy successful".to_string(),
                format!("Deploy of {} for {} successful: {}", tag, repo, result),
            ),
        Err(e) =>
            (
                "Deploy failed".to_string(),
                format!("Deploy of {} for {} failed: {}", tag, repo, e),
            ),
    };

    info!("{body}");
    let _ = mail.read().await.send_mail(subject, body);
}

/// Executes the deployment script.
///
/// # Arguments
///
/// * 'script_path' - Path to the master deployment shell script
/// * 'dev_dir' - Development directory in the server
/// * 'scripts_dir' - Directory where scripts are stored
/// * 'repo_name' - The Git repository name
/// * 'tag' - Git tag to deploy
async fn deploy(script_path: &str, dev_dir: &str, scripts_dir: &str, repo_name: &str, tag: &str) -> Result<String, DeployError> {
    // If you want a simple concurrency guard:
    // use `flock` (recommended on Linux) by invoking it here.
    //
    // Example:
    // Command::new("flock")
    //   .arg("-n").arg("/tmp/deploy.lock")
    //   .arg(script_path).arg(tag) ...

    let out = Command::new(script_path)
        .arg(repo_name)
        .arg(tag)
        .arg(dev_dir)
        .arg(scripts_dir)
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    if out.status.success() {
        Ok(stdout.trim().to_string())
    } else {
        Err(DeployError::CommandExecuteError(format!(
            "exit={:?} stdout={} stderr={}",
            out.status.code(),
            stdout.trim(),
            stderr.trim()
        )))
    }
}

#[derive(Error, Debug)]
pub enum DeployError {
    #[error("CommandSpawnError: {0}")]
    CommandSpawnError(#[from] std::io::Error),
    #[error("CommandExecuteError: {0}")]
    CommandExecuteError(String),
}