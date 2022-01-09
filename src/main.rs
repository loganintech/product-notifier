mod config;
mod error;
mod notifier;
mod scraping;

use chrono::Local;

use config::write_config;
use error::NotifyError;
use notifier::Notifier;

const DEFAULT_DAEMON_TIMEOUT: u64 = 60;

#[tokio::main]
async fn main() -> Result<(), NotifyError> {
    let mut notifier = Notifier::new().await?;
    loop {
        let runtime = match run_bot(&mut notifier).await {
            Ok(runtime) => runtime,
            Err(e) => {
                eprintln!("Error occurred: {:?}", e);
                0
            }
        };

        let wait_time = notifier.config.daemon_timeout.get_or_insert(DEFAULT_DAEMON_TIMEOUT);
        let wait_time = wait_time.saturating_sub(runtime as u64);
        println!("Took {} seconds, waiting {}s.", runtime, wait_time);

        // If we're not in daemon mode, break out of this loop
        if !notifier.config.daemon_mode {
            break;
        }

        // Otherwise, delay for the rest of the 30 second cycle
        tokio::time::delay_for(std::time::Duration::from_secs(wait_time)).await;
    }

    Ok(())
}

async fn run_bot(notifier: &mut Notifier) -> Result<i64, NotifyError> {
    let start = Local::now();
    // Check the scraped websites
    let scraped_set = scraping::get_providers_from_scraping(notifier).await?;

    // Only send a message if we haven't sent one in the last 5 minutes
    for product in scraped_set.iter() {
        // If we found any providers, send the messages
        // If it results in an error print the error
        if let Err(e) = notifier.handle_found_product(product).await {
            eprintln!("Provider {:?} had issue: {:?}", product, e);
        }
    }

    // Once we've run through re-write our config
    write_config(notifier).await?;
    let end = Local::now();
    Ok((end - start).num_seconds())
}
