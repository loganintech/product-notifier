use serde::{Deserialize, Serialize};

use crate::{scraping::target::ScrapingTarget, NotifyError};

pub async fn send_webhook(product: &ScrapingTarget, url: &str) -> Result<(), NotifyError> {
    let message = product.new_stock_message();

    let webhook_body = DiscordWebhook {
        username: Some("RTX Notifier".to_string()),
        avatar_url: Some(
            "https://images.evga.com/products/gallery/png/10G-P5-3897-KR_LG_1.png".to_string(),
        ),
        content: None,
        embeds: vec![WebhookEmbed {
            title: Some(format!("Found Inventory {}", product.key)),
            url: Some(product.url.clone()),
            description: Some(message.clone()),
            color: 0,
            fields: vec![],
        }],
    };

    let payload = serde_json::to_string(&webhook_body).unwrap();

    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .body(payload.clone())
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(NotifyError::WebRequestFailed)?;

    let status = res.status();
    if !status.is_success() || status.is_server_error() || status.is_client_error() {
        return Err(NotifyError::WebClient(status));
    }

    println!("Sent discord webhook to {}\nPayload: {}", message, payload);

    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DiscordWebhook {
    username: Option<String>,
    avatar_url: Option<String>,
    content: Option<String>,
    embeds: Vec<WebhookEmbed>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Author {
    name: Option<String>,
    url: Option<String>,
    icon_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct WebhookEmbed {
    title: Option<String>,
    url: Option<String>,
    description: Option<String>,
    color: usize,
    fields: Vec<EmbedField>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct EmbedField {
    name: Option<String>,
    value: Option<String>,
    inline: Option<bool>,
    thumbnail: Option<Thumbnail>,
    image: Option<Image>,
    footer: Option<Footer>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Thumbnail {
    url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Image {
    url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Footer {
    text: Option<String>,
    icon_url: Option<String>,
}
