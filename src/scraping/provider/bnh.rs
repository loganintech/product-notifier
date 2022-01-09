use async_trait::async_trait;

use crate::scraping::target::ScrapingTarget;
use crate::{error::NotifyError, scraping::ScrapingProvider};

pub struct BnHScraper;

#[async_trait]
impl<'a> ScrapingProvider<'a> for BnHScraper {
    async fn handle_response(
        &'a self,
        resp: reqwest::Response,
        product: &'a ScrapingTarget,
    ) -> Result<&'a ScrapingTarget, NotifyError> {
        let resp = resp.text().await?;

        if resp.contains(r#"showNotifyWhenAvailable":false"#)
            && resp.contains(r#"showNotifyWhenInStock":false"#)
        {
            return Ok(product);
        }

        Err(NotifyError::NoScrapingTargetFound)
    }
}
