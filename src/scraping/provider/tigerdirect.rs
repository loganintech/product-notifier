use std::io::Write;

use async_trait::async_trait;
use regex::{Regex, RegexBuilder};

use crate::{
    error::NotifyError,
    scraping::{ScrapingProvider, target::ScrapingTarget},
};

pub struct TigerDirectScraper;

#[async_trait]
impl<'a> ScrapingProvider<'a> for TigerDirectScraper {
    async fn handle_response(
        &'a self,
        resp: reqwest::Response,
        product: &'a ScrapingTarget,
    ) -> Result<ScrapingTarget, NotifyError> {
        let resp = resp.text().await?;

        if !resp.contains(r#"<h2 class="outofStock">Currently Out Of Stock!</h2>"#) {
            return Ok(product.clone());
        }

        Err(NotifyError::NoScrapingTargetFound)
    }
}
