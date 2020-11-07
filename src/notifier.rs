use std::{collections::HashMap, process::Command};

use chrono::{Duration, Local};

use crate::{
    config::Config, error::NotifyError, notifier::discord::send_webhook,
    scraping::target::ScrapingTarget,
};

mod discord;

pub struct Notifier {
    pub config: Config,
}

impl Notifier {
    pub fn is_ratelimited(&self, key: &str) -> bool {
        if let Some(map) = &self.config.ratelimit_keys {
            match map.get(key) {
                Some(time) => time > &Local::now(),
                _ => false,
            }
        } else { false }
    }

    fn run_command(&self, command: &str, args: &[&str]) -> Result<(), NotifyError> {
        // Run the explorer command with the URL as the param
        let mut child = Command::new(command).args(args).spawn()?;
        let res = child.wait()?;
        if res.success() || res.code() == Some(1) {
            Ok(())
        } else {
            Err(NotifyError::CommandResult(res.code().unwrap_or(0)))
        }
    }

    // If we're using windows
    #[cfg(target_os = "windows")]
    pub fn open_in_browser(&self, url: &str) -> Result<(), NotifyError> {
        self.run_command("cmd", &["/C", "start", url])
    }

    // If we're on a mac
    #[cfg(target_os = "macos")]
    pub fn open_in_browser(&self) -> Result<(), NotifyError> {
        let url = self.get_url()?;
        self.run_command("open", &[url])
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    pub fn open_in_browser(&self) -> Result<(), NotifyError> {
        NotifyError::PlatformNotSupported
    }

    pub async fn handle_found_product(
        &mut self,
        product: &ScrapingTarget,
    ) -> Result<(), NotifyError> {
        // If the notifier is configured to open this in a browser
        if self.config.should_open_browser() {
            // Open the page in a browser
            self.open_in_browser(&product.url)?;
        }

        if let Some(discord_url) = &self.config.discord_url {
            send_webhook(product, discord_url).await?
        }

        Ok(())
    }

    pub fn add_ratelimit(&mut self, product: &ScrapingTarget) {
        if self.config.ratelimit_keys.is_some() {
            self.config
                .ratelimit_keys
                .as_mut()
                .unwrap()
                .insert(product.key.clone(), Local::now() + Duration::minutes(2));
        } else {
            let mut map = std::collections::HashMap::new();
            map.insert(product.key.clone(), Local::now() + Duration::minutes(2));
            self.config.ratelimit_keys = Some(map);
        }
    }
}
