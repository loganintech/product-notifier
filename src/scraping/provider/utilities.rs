use reqwest::header::HeaderMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::error::NotifyError;

#[allow(dead_code)]
pub async fn write_response_to_file<'a, T: Into<&'a [u8]>>(
    resp: T,
    folder: &str,
    product_name: &str,
    headers: Option<HeaderMap>,
) -> Result<(), NotifyError> {
    // Ignore the error here, since it'll error if the folder already exists
    let _ = tokio::fs::DirBuilder::new().recursive(true).create(format!("./logs/{}", folder)).await;
    let mut file = tokio::fs::File::create(format!(
        "./logs/{}/{}-log-{}.txt",
        folder,
        product_name,
        chrono::Local::now().to_rfc3339().replace(":", "-"),
    ))
        .await?;

    if let Some(map) = headers {
        for header in map {
            let header = match header {
                (Some(name), val) => {
                    let mut bytes = Vec::new();
                    name.as_str()
                        .as_bytes()
                        .chain(val.as_bytes())
                        .read_to_end(&mut bytes).await?;
                    bytes
                }
                _ => continue,
            };
            file.write_all(header.as_slice()).await?;
        }
    }

    file.write_all(resp.into()).await?;

    Ok(())
}