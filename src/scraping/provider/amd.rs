use async_trait::async_trait;

use crate::{
    error::NotifyError,
    scraping::{target::ScrapingTarget, ScrapingProvider},
};

pub struct AmdScraper;

#[async_trait]
impl<'a> ScrapingProvider<'a> for AmdScraper {
    async fn handle_response(
        &'a self,
        resp: reqwest::Response,
        product: &'a ScrapingTarget,
    ) -> Result<&'a ScrapingTarget, NotifyError> {
        let resp = resp.text().await?;

        if !resp.contains(r#"<p class="product-out-of-stock">Out of stock</p>"#) {
            return Ok(product);
        }

        Err(NotifyError::NoScrapingTargetFound)
    }
}
