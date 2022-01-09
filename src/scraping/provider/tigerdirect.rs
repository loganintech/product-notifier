use async_trait::async_trait;

use crate::{
    error::NotifyError,
    scraping::{target::ScrapingTarget, ScrapingProvider},
};

pub struct TigerDirectScraper;

#[async_trait]
impl<'a> ScrapingProvider<'a> for TigerDirectScraper {
    async fn handle_response(
        &'a self,
        resp: reqwest::Response,
        product: &'a ScrapingTarget,
    ) -> Result<&'a ScrapingTarget, NotifyError> {
        let resp = resp.text().await?;

        if !resp.contains(r#"<h2 class="outofStock">Currently Out Of Stock!</h2>"#) {
            return Ok(product);
        }

        Err(NotifyError::NoScrapingTargetFound)
    }
}
