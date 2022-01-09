use async_trait::async_trait;

use crate::{error::NotifyError, scraping::ScrapingProvider};
use crate::scraping::target::ScrapingTarget;



pub struct BnHScraper;

#[async_trait]
impl<'a> ScrapingProvider<'a> for BnHScraper {
    async fn handle_response(
        &'a self,
        resp: reqwest::Response,
        product: &'a ScrapingTarget,
    ) -> Result<ScrapingTarget, NotifyError> {
        let resp = resp.text().await?;

        if resp.contains(r#"showNotifyWhenAvailable":false"#)
            && resp.contains(r#"showNotifyWhenInStock":false"#)
        {
            return Ok(product.clone());
        }

        Err(NotifyError::NoScrapingTargetFound)
    }
}
