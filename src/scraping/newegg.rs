use std::io::Write;

use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};

use crate::{
    error::NotifyError,
    scraping::{target::ScrapingTarget, ScrapingProvider},
};

lazy_static! {
    // Look for the javascript tag that loads the raw product.rs data from their webservers
    static ref DETAIL_REGEX: Regex =
        Regex::new(r#"<script type="text/javascript" src="(.+ItemInfo4.+)">"#).unwrap();

    static ref INSTOCK_REGEX: Regex = RegexBuilder::new(r#""instock":true"#)
        .case_insensitive(true).ignore_whitespace(true).build().unwrap();
}

pub struct NeweggScraper;

#[async_trait]
impl<'a> ScrapingProvider<'a> for NeweggScraper {
    async fn handle_response(
        &'a self,
        resp: reqwest::Response,
        product: &'a ScrapingTarget,
    ) -> Result<ScrapingTarget, NotifyError> {
        let resp = resp.text().await?;

        if resp.contains("We apologize for the confusion, but we can't quite tell if you're a person or a script.") {
            return Err(NotifyError::RateLimit);
        }

        // A new version doesn't call the script anymore, they just load the entire window property directly into the HTML
        if INSTOCK_REGEX.is_match(&resp) {
            return Ok(product.clone());
        }

        let capture = DETAIL_REGEX.captures_iter(&resp).next();
        // If we found the js tag with the detail URL, act on it
        if let Some(capture) = capture {
            // Extract the URL knowing capture[0] is the entire match, not just the capturing group
            let product_url = &capture[1];

            // And load the product.rs url
            let product_resp = reqwest::get(product_url).await?.text().await?;

            // Then look for the JSON property that shows it's in stock. Yes, we could serialize this but why bother right now
            if INSTOCK_REGEX.is_match(&product_resp) {
                return Ok(product.clone());
            }
        }

        Err(NotifyError::NoScrapingTargetFound)
    }
}

#[allow(dead_code)]
fn write_newegg_log<'a, T: Into<&'a [u8]>>(resp: T) -> Result<(), NotifyError> {
    let mut file = std::fs::File::create(format!(
        "./newegg_log/newegg-log-{}.txt",
        chrono::Local::now().to_rfc3339().replace(":", "-"),
    ))?;

    Ok(file.write_all(resp.into())?)
}
