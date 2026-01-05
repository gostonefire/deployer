use subtle::ConstantTimeEq;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use hmac::{Hmac, Mac};
use log::{error, info};
use serde::Deserialize;
use sha2::Sha256;
use crate::AppState;
use crate::deployer::run_deploy;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Deserialize)]
struct PushEvent {
    #[serde(default)]
    r#ref: String, // e.g. "refs/tags/v1.2.3"
    #[serde(default)]
    deleted: bool,
    #[serde(default)]
    repository: Repository,
}

#[derive(Debug, Default, Deserialize)]
struct Repository {
    #[serde(default)]
    full_name: String,
}

pub async fn github_webhook(State(state): State<AppState>, headers: HeaderMap, body: Bytes) -> impl IntoResponse {

    // 1) Verify event type
    let event = headers
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if event != "push" {
        return (StatusCode::OK, "ignored (not push)").into_response();
    }

    // 2) Verify signature (X-Hub-Signature-256)
    let sig_header = match headers.get("X-Hub-Signature-256").and_then(|v| v.to_str().ok()) {
        Some(s) => s,
        None => return (StatusCode::UNAUTHORIZED, "missing signature").into_response(),
    };

    if !verify_github_signature(&state.webhook_secret, &body, sig_header) {
        return (StatusCode::UNAUTHORIZED, "bad signature").into_response();
    }

    // 3) Parse JSON after signature check
    let payload: PushEvent = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            error!("invalid json: {e}");
            return (StatusCode::BAD_REQUEST, "invalid json").into_response();
        }
    };

    // 4) Validate repo (recommended)
    if !state.expected_repo_full_name.contains(&payload.repository.full_name) {
        error!("repo not ok for auto deploy: got={}", payload.repository.full_name);
        return (StatusCode::FORBIDDEN, "repo mismatch").into_response();
    }

    // 5) Only deploy on tag creation
    // Tag pushes come as: refs/tags/<tag>
    const TAG_PREFIX: &str = "refs/tags/";
    if !payload.r#ref.starts_with(TAG_PREFIX) {
        return (StatusCode::OK, "ignored (not a tag ref)").into_response();
    }
    if payload.deleted {
        return (StatusCode::OK, "ignored (tag deleted)").into_response();
    }

    let tag = payload.r#ref[TAG_PREFIX.len()..].to_string();
    info!("deploy requested for repo: {} with tag: {}", payload.repository.full_name, tag);

    // 6) Run deploy script
    tokio::spawn(run_deploy(
        state.deploy_script_path,
        state.dev_dir,
        state.script_log_dir,
        payload.repository.full_name,
        tag.clone(),
        state.mail.clone()));

    (StatusCode::OK, "deploy triggered").into_response()
}

fn verify_github_signature(secret: &str, body: &[u8], sig_header: &str) -> bool {
    // GitHub format: "sha256=<hex>"
    let provided = sig_header.strip_prefix("sha256=").unwrap_or("");
    if provided.len() != 64 {
        return false;
    }

    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };

    mac.update(body);
    let expected = mac.finalize().into_bytes(); // 32 bytes

    let provided_bytes = match hex::decode(provided) {
        Ok(b) => b,
        Err(_) => return false,
    };
    if provided_bytes.len() != expected.len() {
        return false;
    }

    expected.ct_eq(&provided_bytes).into()
}
