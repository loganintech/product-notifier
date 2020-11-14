use std::process::Command;

use chrono::{Duration, Local};

use discord::{DiscordWebhook, WebhookEmbed};

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
            if let Some(time) = map.get(key) {
                return time > &Local::now();
            }
        }

        false
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
    pub fn open_in_browser(&self, url: &str) -> Result<(), NotifyError> {
        Err(NotifyError::PlatformNotSupported)
    }

    pub async fn handle_test_product_not_found(
        &mut self,
        product: &ScrapingTarget,
    ) -> Result<(), NotifyError> {
        let webhook_body = DiscordWebhook {
            username: Some("Product Notifier".to_string()),
            avatar_url: Some(
                "https://www.amd.com/system/files/styles/992px/private/2020-09/616656-amd-ryzen-9-5000-series-PIB-1260x709_0.png".to_string(),
            ),
            content: None,
            embeds: vec![WebhookEmbed {
                title: Some(format!("Test Product [{}] not In Stock at {}", product.name, product.key)),
                url: Some(product.url.clone()),
                description: Some("Test product was not in stock, if the link shows it is in stock there's probably a bug with the provider implementation.".to_string()),
                color: 0,
                fields: vec![],
            }],
        };

        if let Some(discord_url) = &self.config.discord_url {
            send_webhook(discord_url, webhook_body).await?
        }

        Ok(())
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

        let message = product.new_stock_message();

        let webhook_body = DiscordWebhook {
            username: Some("Product Notifier".to_string()),
            avatar_url: Some(
                "https://www.amd.com/system/files/styles/992px/private/2020-09/616656-amd-ryzen-9-5000-series-PIB-1260x709_0.png".to_string(),
            ),
            content: None,
            embeds: vec![WebhookEmbed {
                title: Some(format!("Found Product [{}] {}", product.name, product.key)),
                url: Some(product.url.clone()),
                description: Some(message.clone()),
                color: 0,
                fields: vec![],
            }],
        };

        if let Some(discord_url) = &self.config.discord_url {
            send_webhook(discord_url, webhook_body).await?
        }

        Ok(())
    }

    pub fn add_ratelimit(&mut self, product: &ScrapingTarget) {
        let mut map = self.config.ratelimit_keys.take().unwrap_or_default();
        map.insert(product.key.clone(), Local::now() + Duration::minutes(2));
        self.config.ratelimit_keys = Some(map);
    }
}
