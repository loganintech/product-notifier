use std::collections::{HashMap, HashSet};
use std::time::Duration;

use async_trait::async_trait;
use regex::Regex;
use reqwest::header::HeaderMap;

pub use provider::{amazon, amd, antonline, bestbuy, bnh, newegg, tigerdirect};
use target::ScrapingTarget;

use crate::{error::NotifyError, notifier::Notifier};

mod provider;
pub mod target;

#[async_trait]
pub trait ScrapingProvider<'a> {
    fn absent_regex(&self) -> Option<&Regex> {
        None
    }

    async fn get_request(
        &'a self,
        product: &'a ScrapingTarget,
        client: &reqwest::Client,
    ) -> Result<reqwest::Response, NotifyError> {
        // Load the webpage
        client
            .get(&product.url)
            .send()
            .await
            .map_err(NotifyError::WebRequestFailed)
    }

    async fn handle_response(
        &'a self,
        resp: reqwest::Response,
        product: &'a ScrapingTarget,
    ) -> Result<&'a ScrapingTarget, NotifyError> {
        let resp = resp.text().await?;

        self.absent_regex()
            .map(|reg| {
                if !reg.is_match(&resp) {
                    return Ok(product);
                } else {
                    Err(NotifyError::NoScrapingTargetFound)
                }
            })
            .unwrap_or_else(|| Err(NotifyError::DefaultResponseHandlerMissingRegex))
    }

    async fn is_available(
        &'a self,
        product: &'a ScrapingTarget,
        client: &reqwest::Client,
    ) -> Result<&'a ScrapingTarget, NotifyError> {
        let resp = self.get_request(product, client).await?;
        let status = resp.status();

        // If we're being rate limited
        //https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/429
        if status.as_u16() == 429
            || (product.key == "bnh"
                && resp.url().as_str() == "https://site-not-available.bhphotovideo.com/500Error")
        {
            return Err(NotifyError::RateLimit);
        }

        if status.is_server_error() {
            return Err(NotifyError::WebServer(status));
        }

        if status.is_client_error() {
            return Err(NotifyError::WebClient(status));
        }

        if !status.is_success() {
            return Err(NotifyError::BadStatus(status));
        }

        self.handle_response(resp, product).await
    }
}

pub async fn get_providers_from_scraping(
    notifier: &mut Notifier,
) -> Result<HashSet<ScrapingTarget>, NotifyError> {
    let client = get_client(&notifier)?;
    let active_products = notifier
        .config
        .targets
        .iter()
        .filter(|p| p.is_active() && !notifier.is_ratelimited(&p.key))
        .cloned()
        .collect::<Vec<ScrapingTarget>>();

    let mut futs = vec![];
    for product in &active_products {
        futs.push(product.is_available(&client));
    }
    let joined = futures::future::join_all(futs).await;

    let mut should_reload_tor = false;
    let mut checked: HashMap<String, (usize, Vec<String>)> = HashMap::new();
    let mut providers = HashSet::new();
    for (i, res) in joined.into_iter().enumerate() {
        let product = &active_products[i];
        match res {
            Ok(res) => {
                // If it's a test product, don't make a notification for it
                modify_checked_map(product, &mut checked);
                if matches!(res.is_test, Some(true)) {
                    continue;
                }
                if !providers.insert(res) {
                    eprintln!("Duplicate provider found.");
                }
            }
            Err(NotifyError::RateLimit) => {
                should_reload_tor = true;
                print_err(product, NotifyError::RateLimit);
                notifier.add_ratelimit(&product);
            }
            Err(NotifyError::WebRequestFailed(e)) => print_err(product, e),
            Err(NotifyError::NoScrapingTargetFound) => {
                modify_checked_map(product, &mut checked);
                // If it's a test product and it's not found, make sure we notify that a potential bug exists
                if matches!(product.is_test, Some(true)) {
                    if let Err(e) = notifier.handle_test_product_not_found(product).await {
                        print_err(product, e);
                    }
                }
            }
            Err(e) => print_err(product, e),
        }
    }

    if should_reload_tor {
        #[cfg(target_os = "linux")]
        if let Err(e) = reload_tor() {
            eprintln!("Error reloading TOR: {}", e);
        }
    }

    println!("Sites Checked:");
    for (key, (count, list)) in checked.keys().zip(checked.values()) {
        println!("[{:02}] {}: {:?}", count, key, list);
    }

    Ok(providers
        .into_iter()
        .cloned()
        .collect::<std::collections::HashSet<ScrapingTarget>>())
}

fn get_client(notifier: &Notifier) -> Result<reqwest::Client, NotifyError> {
    let proxy_url = &notifier.config.proxy_url;
    let mut headers = HeaderMap::new();
    headers.insert(
        reqwest::header::ACCEPT,
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8"
            .parse()
            .unwrap(),
    );
    headers.insert(
        reqwest::header::ACCEPT_LANGUAGE,
        "en-US,en;q=0.5".parse().unwrap(),
    );
    let mut client_builder = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(
            notifier.config.daemon_timeout.unwrap_or_else(|| 30),
        ))
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:82.0) Gecko/20100101 Firefox/82.0",
        )
        .gzip(true);
    if let Some(proxy_url) = proxy_url {
        let proxy = reqwest::Proxy::all(proxy_url).map_err(|_| NotifyError::ProxyNotRunning)?;
        client_builder = client_builder.proxy(proxy);
    }
    let client = client_builder
        .build()
        .map_err(|e| NotifyError::ClientBuild(e))?;

    Ok(client)
}

fn modify_checked_map(product: &ScrapingTarget, map: &mut HashMap<String, (usize, Vec<String>)>) {
    let name = product.name.to_string();
    map.entry(product.key.clone())
        .and_modify(|(count, products)| {
            *count += 1;
            products.push(name.clone())
        })
        .or_insert((1, vec![name]));
}

#[cfg(target_os = "linux")]
fn reload_tor() -> Result<(), NotifyError> {
    let mut child = std::process::Command::new("service")
        .args(&["tor", "reload"])
        .spawn()?;
    let res = child.wait()?;
    if res.success() {
        Ok(())
    } else {
        Err(NotifyError::CommandResult(res.code().unwrap_or(0)))
    }
}

fn print_err(product: &ScrapingTarget, e: impl std::error::Error) {
    eprintln!(
        "==========\nError Happened: {}\n====\nWith Product: {:?}\n==========",
        e, product
    );
}
