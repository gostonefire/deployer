use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use anyhow::Result;
use axum::Router;
use axum::routing::post;
use axum_server::tls_rustls::RustlsConfig;
use log::info;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use crate::handlers::github_webhook;
use crate::initialization::config;
use crate::manager_mail::Mail;

mod initialization;
mod logging;
mod manager_mail;
mod handlers;

struct Comms {
    tx_to_dispatcher: UnboundedSender<String>,
    rx_from_dispatcher: UnboundedReceiver<String>,
}

#[derive(Clone)]
struct AppState {
    comms: Arc<Mutex<Comms>>,
    mail: Arc<RwLock<Mail>>,
    webhook_secret: String,
    expected_repo_full_name: String, // e.g. "yourorg/yourrepo"
    deploy_script_path: String,       // e.g. "/opt/deploy/deploy.sh"
}

#[tokio::main]
async fn main() -> Result<(), UnrecoverableError> {
    // Set up communication channels
    let (mut tx_to_dispatcher, mut rx_from_web) = mpsc::unbounded_channel::<String>();
    let (mut tx_to_web, mut rx_from_dispatcher) = mpsc::unbounded_channel::<String>();
    let comms = Arc::new(Mutex::new(Comms{tx_to_dispatcher,rx_from_dispatcher,}));

    // Load configuration
    let (config, mail) = config()?;
    let mail = Arc::new(RwLock::new(mail));

    // Print version
    info!("deployer version: {}", env!("CARGO_PKG_VERSION"));

    // Web server
    info!("starting web server");
    let shared_state = AppState {
        comms: comms.clone(), 
        mail: mail.clone(), 
        webhook_secret: "".to_string(), 
        expected_repo_full_name: "".to_string(), 
        deploy_script_path: "".to_string(),
    };

    let app = Router::new()
        .route("/deploy", post(github_webhook))
        .with_state(shared_state);

    let ip_addr = Ipv4Addr::from_str(&config.web_server.bind_address).expect("invalid BIND_ADDR");
    let addr = SocketAddr::new(IpAddr::V4(ip_addr), config.web_server.bind_port);

    let rustls_config = RustlsConfig::from_pem_file(&config.web_server.tls_chain_cert, &config.web_server.tls_private_key)
        .await?;

    tokio::spawn(axum_server::bind_rustls(addr, rustls_config)
        .serve(app.into_make_service()));

    // Main dispatch function
    info!("starting main dispatch function");
    loop {
        run(tx_to_web, rx_from_web, &config).await;

        info!("restarting main dispatch function");
        (tx_to_dispatcher, rx_from_web) = mpsc::unbounded_channel::<String>();
        (tx_to_web, rx_from_dispatcher) = mpsc::unbounded_channel::<String>();
        {
            let mut disp_comms = comms.lock().await;
            disp_comms.tx_to_dispatcher = tx_to_dispatcher;
            disp_comms.rx_from_dispatcher = rx_from_dispatcher;
        }
    }
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
