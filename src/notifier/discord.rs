use serde::{Deserialize, Serialize};

use crate::NotifyError;

pub async fn send_webhook(url: &str, webhook_body: DiscordWebhook) -> Result<(), NotifyError> {
    let payload = serde_json::to_string(&webhook_body).unwrap();

    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .body(payload)
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(NotifyError::WebRequestFailed)?;

    let status = res.status();
    if !status.is_success() || status.is_server_error() || status.is_client_error() {
        return Err(NotifyError::WebClient(status));
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiscordWebhook {
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub content: Option<String>,
    pub embeds: Vec<WebhookEmbed>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Author {
    pub name: Option<String>,
    pub url: Option<String>,
    pub icon_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebhookEmbed {
    pub title: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub color: usize,
    pub fields: Vec<EmbedField>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmbedField {
    pub name: Option<String>,
    pub value: Option<String>,
    pub inline: Option<bool>,
    pub thumbnail: Option<Thumbnail>,
    pub image: Option<Image>,
    pub footer: Option<Footer>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Thumbnail {
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Image {
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Footer {
    pub text: Option<String>,
    pub icon_url: Option<String>,
}
