use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use anyhow::Result;
use axum::Router;
use axum::routing::post;
use log::info;
use tokio::sync::RwLock;
use crate::handlers::github_webhook;
use crate::initialization::config;
use crate::manager_mail::Mail;

mod initialization;
mod logging;
mod manager_mail;
mod handlers;
mod deployer;

#[derive(Clone)]
struct AppState {
    mail: Arc<RwLock<Mail>>,
    webhook_secret: String,
    expected_repo_full_name: Vec<String>, // e.g. "yourorg/yourrepo"
    deploy_script_path: String,       // e.g. "/opt/deploy/deploy.sh"
}

#[tokio::main]
async fn main() -> Result<(), UnrecoverableError> {
    // Load configuration
    let (config, mail) = config()?;
    let mail = Arc::new(RwLock::new(mail));

    // Print version
    info!("deployer version: {}", env!("CARGO_PKG_VERSION"));

    // Web server
    info!("starting web server");
    let shared_state = AppState {
        mail: mail.clone(),
        webhook_secret: config.github.webhook_secret.clone(), 
        expected_repo_full_name: config.github.expected_repo_full_name.clone(), 
        deploy_script_path: config.github.deploy_script_path.clone(),
    };

    let app = Router::new()
        .route("/deploy", post(github_webhook))
        .with_state(shared_state);

    let ip_addr = Ipv4Addr::from_str(&config.web_server.bind_address).expect("invalid BIND_ADDR");
    let addr = SocketAddr::new(IpAddr::V4(ip_addr), config.web_server.bind_port);

    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/// Error depicting errors that are not recoverable
///
#[derive(Debug, Error)]
pub enum UnrecoverableError {
    #[error("InitializationError: {0}")]
    InitializationError(#[from] initialization::InitializationError),
    #[error("RustlsConfigError: {0}")]
    RustlsConfigError(#[from] std::io::Error),
    #[error("MailError: {0}")]
    MailError(#[from] manager_mail::MailError),
}
