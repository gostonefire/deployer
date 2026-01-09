use std::{env, fs};
use std::path::PathBuf;
use log::LevelFilter;
use serde::Deserialize;
use thiserror::Error;
use crate::logging::{setup_logger, LoggerError};
use crate::manager_mail::{Mail, MailError};

#[derive(Deserialize)]
pub struct WebServerParameters {
    pub bind_address: String,
    pub bind_port: u16,
}

#[derive(Deserialize)]
pub struct MailParameters {
    #[serde(default)]
    pub smtp_user: String,
    #[serde(default)]
    pub smtp_password: String,
    pub smtp_endpoint: String,
    pub from: String,
    pub to: String,
}

#[derive(Deserialize)]
pub struct GitHub {
    #[serde(default)]
    pub webhook_secret: String,
    pub expected_repo_full_name: Vec<String>,
    pub deploy_script_path: String,
}
#[derive(Deserialize)]
pub struct General {
    pub dev_dir: String,
    pub scripts_dir: String,
    pub log_path: String,
    pub log_level: LevelFilter,
    pub log_to_stdout: bool,
}

#[derive(Deserialize)]
pub struct Config {
    pub web_server: WebServerParameters,
    pub mail: MailParameters,
    pub github: GitHub,
    pub general: General,
}

/// Returns a configuration struct for the application and starts logging
///
pub fn config() -> Result<(Config, Mail), InitializationError> {
    let args: Vec<String> = env::args().collect();
    let config_path = args.iter()
        .find(|p| p.starts_with("--config="))
        .ok_or(InitializationError::ConfigFilePathError("missing --config=<config_path>".to_string()))?;
    let config_path = config_path
        .split_once('=')
        .ok_or(InitializationError::ConfigFilePathError("invalid --config=<config_path>".to_string()))?
        .1;
    
    let mut config = load_config(&config_path)?;
    config.mail.smtp_user = read_credential("mail_smtp_user")?;
    config.mail.smtp_password = read_credential("mail_smtp_password")?;
    config.github.webhook_secret = read_credential("github_webhook_secret")?;
    
    setup_logger(&config.general.log_path, config.general.log_level, config.general.log_to_stdout)?;
    
    let mail = Mail::new(&config.mail)?;
    
    Ok((config, mail))
}

/// Loads the configuration file and returns a struct with all configuration items
///
/// # Arguments
///
/// * 'config_path' - path to the config file
fn load_config(config_path: &str) -> Result<Config, InitializationError> {

    let toml = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&toml)?;

    Ok(config)
}

/// Reads a credential from the file system supported by the credstore and
/// given from systemd
///
/// # Arguments
///
/// * 'name' - name of the credential to read
fn read_credential(name: &str) -> Result<String, InitializationError> {
    let dir = env::var("CREDENTIALS_DIRECTORY")?;
    let mut p = PathBuf::from(dir);
    p.push(name);
    let bytes = fs::read(p)?;
    Ok(String::from_utf8(bytes)?.trim_end().to_string())
}

/// Error depicting errors that occur while initializing the scheduler
///
#[derive(Debug, Error)]
pub enum InitializationError {
    #[error("FileIOError: {0}")]
    FileIOError(#[from] std::io::Error),
    #[error("ConfigFileError: {0}")]
    ConfigFileError(#[from] toml::de::Error),
    #[error("ConfigurationError: {0}")]
    ConfigFilePathError(String),
    #[error("SetupLoggerError: {0}")]
    SetupLoggerError(#[from] LoggerError),
    #[error("MailSetupError: {0}")]
    MailSetupError(#[from] MailError),
    #[error("CredentialEnvError: {0}")]
    CredentialEnvError(#[from] env::VarError),
    #[error("CredentialUtf8Error: {0}")]
    CredentialUtf8Error(#[from] std::string::FromUtf8Error),
}
