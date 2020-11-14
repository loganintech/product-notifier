use std::io::Write;

use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};

use crate::{
    error::NotifyError,
    scraping::{ScrapingProvider, target::ScrapingTarget},
};

lazy_static! {
    static ref SOLD_OUT_BUTTON: Regex = RegexBuilder::new("Sold Out\\s?</span")
        .case_insensitive(true)
        .build()
        .unwrap();
}

pub struct AntScraper;

#[async_trait]
impl<'a> ScrapingProvider<'a> for AntScraper {
    async fn handle_response(
        &'a self,
        resp: reqwest::Response,
        product: &'a ScrapingTarget,
    ) -> Result<ScrapingTarget, NotifyError> {
        let resp = resp.text().await?;

        if !SOLD_OUT_BUTTON.is_match(&resp) {
            return Ok(product.clone());
        }

        Err(NotifyError::NoScrapingTargetFound)
    }
}
