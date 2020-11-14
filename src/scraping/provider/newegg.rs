use std::io::Write;

use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};

use crate::{
    error::NotifyError,
    scraping::{ScrapingProvider, target::ScrapingTarget},
};

lazy_static! {
    // Look for the javascript tag that loads the raw product.rs data from their webservers
    static ref DETAIL_REGEX: Regex =
        Regex::new(r#"<script type="text/javascript" src="(.+ItemInfo4.+)">"#).unwrap();

    static ref INSTOCK_REGEX: Regex = RegexBuilder::new(r#""instock":([a-zA-Z]+)"#)
        .case_insensitive(true).ignore_whitespace(true).build().unwrap();
    static ref SELLERNAME_REGEX: Regex = RegexBuilder::new(r#""sellername":"?([a-zA-Z0-9]+)"?"#)
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
        if has_stock(&resp) {
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
            if has_stock(&product_resp) {
                return Ok(product.clone());
            }
        }

        Err(NotifyError::NoScrapingTargetFound)
    }
}

fn has_stock(page_data: &str) -> bool {
    let seller_list = SELLERNAME_REGEX
        // Get the list of captures
        .captures_iter(&page_data)
        .filter_map(|a| {
            // Get the name group directly
            a.get(1).map(|cap| cap.as_str())
        })
        .collect::<Vec<&str>>();

    let in_stock_list = INSTOCK_REGEX
        // Get the list of captures
        .captures_iter(&page_data)
        .filter_map(|a| {
            // Get the name group directly
            a.get(1).map(|cap| cap.as_str())
        })
        .skip(1)
        .collect::<Vec<&str>>();

    seller_list
        .iter()
        .zip(in_stock_list.iter())
        .any(|(seller, has_stock)| seller == &"null" && has_stock == &"true")
}

#[allow(dead_code)]
fn write_newegg_log<'a, T: Into<&'a [u8]>>(resp: T) -> Result<(), NotifyError> {
    let mut file = std::fs::File::create(format!(
        "./newegg_log/newegg-log-{}.txt",
        chrono::Local::now().to_rfc3339().replace(":", "-"),
    ))?;

    Ok(file.write_all(resp.into())?)
}
