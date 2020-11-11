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
        self.active.unwrap_or_else(|| true)
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
