use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use reqwest::header::HeaderMap;

use crate::{error::NotifyError, scraping::target::ScrapingTarget, scraping::ScrapingProvider};

// Look for the div that says it's Sold Out, case insensitive. Give it a bit of before and after HTML so that it doesn't false match on other elements
lazy_static! {
    static ref BUTTON_REGEX: Regex = RegexBuilder::new(r#"Sold Out</button>"#)
        .case_insensitive(true)
        .build()
        .expect("Invalid regex");
}

pub struct BestBuyScraper;

#[async_trait]
impl<'a> ScrapingProvider<'a> for BestBuyScraper {
    fn absent_regex(&self) -> Option<&Regex> {
        Some(&BUTTON_REGEX)
    }

    async fn get_request(
        &'a self,
        product: &'a ScrapingTarget,
        client: &reqwest::Client,
    ) -> Result<reqwest::Response, NotifyError> {
        // Create a new client, can't use the reqwest::get() because we need headers
        let mut headers = HeaderMap::new();
        // Add some headers for a user agent, otherwise the host refuses connection
        headers.insert("User-Agent", "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:80.0) Gecko/20100101 Firefox/80.0".parse().unwrap());

        // Load the webpage
        client
            .get(&product.url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| NotifyError::ClientBuild(e))
    }
}
