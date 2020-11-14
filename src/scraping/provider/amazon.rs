use std::io::prelude::*;

use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use reqwest::header::HeaderMap;
use tokio::io::AsyncWriteExt;

use crate::{error::NotifyError, scraping::ScrapingProvider, scraping::target::ScrapingTarget};

lazy_static! {
    // See if it's offering us a sale on another seller
    static ref OTHER_SELLER_REGEX: Regex =
        RegexBuilder::new("Available from .+these sellers</a>").case_insensitive(true).build().unwrap();
}

static CAPTCHA_TEXT: &str = r#"<p class="a-last">Sorry, we just need to make sure you're not a robot. For best results, please make sure your browser is accepting cookies.</p>"#;

pub struct AmazonScraper;

#[async_trait]
impl<'a> ScrapingProvider<'a> for AmazonScraper {
    async fn handle_response(
        &'a self,
        resp: reqwest::Response,
        product: &'a ScrapingTarget,
    ) -> Result<ScrapingTarget, NotifyError> {
        let _headers = resp.headers().clone();
        let resp_text = resp.text().await?;

        if resp_text.contains(CAPTCHA_TEXT) {
            return Err(NotifyError::RateLimit);
        }

        if !resp_text.contains(r#"Currently unavailable.</span>"#)
            && !OTHER_SELLER_REGEX.is_match(&resp_text)
        {
            return Ok(product.clone());
        }

        Err(NotifyError::NoScrapingTargetFound)
    }
}
