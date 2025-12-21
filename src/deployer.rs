use std::sync::Arc;
use tokio::process::Command;
use anyhow::Result;
use log::{error, info};
use thiserror::Error;
use tokio::sync::RwLock;
use crate::manager_mail::Mail;

pub async fn run_deploy(script_path: String, repo: String, tag: String, mail: Arc<RwLock<Mail>>) {
    let repo_name = repo.rsplit_once('/').map(|(_, name)| name).unwrap_or(&repo);
    let full_path = format!("{}/{}", script_path, repo_name);
    
    let (subject, body) = match deploy_mock(&full_path, &tag).await {
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

async fn deploy_mock(full_path: &str, tag: &str) -> Result<String, DeployError> {
    info!("running script {} for tag {}", full_path, tag);
    
    Ok(format!("mock deploy result for {}-{}", full_path, tag))
}

async fn deploy(script_path: &str, tag: &str) -> Result<String, DeployError> {
    // If you want a simple concurrency guard:
    // use `flock` (recommended on Linux) by invoking it here.
    //
    // Example:
    // Command::new("flock")
    //   .arg("-n").arg("/tmp/deploy.lock")
    //   .arg(script_path).arg(tag) ...

    let out = Command::new(script_path)
        .arg(tag)
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