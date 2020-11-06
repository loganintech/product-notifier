use std::collections::HashMap;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncReadExt, io::AsyncWriteExt};

use crate::error::NotifyError;
use crate::notifier::Notifier;
use crate::scraping::target::ScrapingTarget;

const CONFIG_FILE_PATH: &str = "./config.json";

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Config {
    pub should_open_browser: Option<bool>,
    pub daemon_mode: bool,
    pub daemon_timeout: Option<u64>,
    pub discord_url: Option<String>,
    pub ratelimit_keys: Option<HashMap<String, DateTime<Local>>>,
    pub proxy_url: Option<String>,
    pub targets: Vec<ScrapingTarget>,
}

impl Config {
    pub fn should_open_browser(&self) -> bool {
        self.should_open_browser.unwrap_or(false)
    }
}

impl Notifier {
    pub async fn new() -> Result<Self, NotifyError> {
        // Open our config
        let mut file = match File::open(CONFIG_FILE_PATH).await {
            Err(_) => {
                return Ok(Notifier {
                    config: Config::default(),
                })
            }
            Ok(file) => file,
        };

        // And read it into a string
        let mut buf = String::new();
        file.read_to_string(&mut buf).await?;

        // Use serde to deserialize the config
        let config: Config = serde_json::from_str(&buf)?;

        // And return our built notifier
        Ok(Notifier { config })
    }
}

pub async fn write_config(notifier: &mut Notifier) -> Result<(), NotifyError> {
    // Open the config file, creating it if it doesn't exist
    let mut file = File::create(CONFIG_FILE_PATH).await?;

    // Write the config file json back in a pretty editable format
    Ok(file
        .write_all(serde_json::to_string_pretty(&notifier.config)?.as_bytes())
        .await?)
}
