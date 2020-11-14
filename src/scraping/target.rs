use chrono::{Local, Timelike};
use serde::{Deserialize, Serialize};

use crate::error::NotifyError;
use crate::scraping::ScrapingProvider;

use super::{
    amazon::AmazonScraper, amd::AmdScraper, antonline::AntScraper, bestbuy::BestBuyScraper,
    bnh::BnHScraper, newegg::NeweggScraper,
};

#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
pub struct ScrapingTarget {
    pub name: String,
    pub url: String,
    pub key: String,
    active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_test: Option<bool>,
}

impl ScrapingTarget {
    // Get some new in stock messages depending on product.rs type
    pub fn new_stock_message(&self) -> String {
        format!(
            "{} has new {} for sale at {} !",
            self.key, self.name, self.url
        )
    }

    pub fn is_active(&self) -> bool {
        match (self.active, self.is_test) {
            // If it's active and a test, check our time
            (Some(true), Some(true)) => {
                // If we're in debug mode OR If the time is divisible by 10, AKA, every 10 minutes, check a test product
                cfg!(debug_assertions) || Local::now().minute() % 10 == 0
            }
            // If it's active and not our test, just go for it
            (Some(true), _) => true,
            // If it's not active, ignore it
            _ => false,
        }
    }

    pub async fn is_available(
        &self,
        client: &reqwest::Client,
    ) -> Result<ScrapingTarget, NotifyError> {
        match self.key.as_str() {
            "newegg" => NeweggScraper.is_available(self, client).await,
            "amazon" => AmazonScraper.is_available(self, client).await,
            "bestbuy" => BestBuyScraper.is_available(self, client).await,
            "bnh" => BnHScraper.is_available(self, client).await,
            "antonline" => AntScraper.is_available(self, client).await,
            "amd" => AmdScraper.is_available(self, client).await,
            _ => Err(NotifyError::NoScrapingTargetFound),
        }
    }
}
