use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};

use crate::scraping::ScrapingProvider;

lazy_static! {
    static ref SOLD_OUT_BUTTON: Regex = RegexBuilder::new("Sold Out\\s?</span")
        .case_insensitive(true)
        .build()
        .unwrap();
}

pub struct AntScraper;

#[async_trait]
impl<'a> ScrapingProvider<'a> for AntScraper {
    fn absent_regex(&self) -> Option<&Regex> {
        Some(&SOLD_OUT_BUTTON)
    }
}
