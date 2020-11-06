use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotifyError {
    #[error("Browser could not be opened.")]
    BrowserOpenError,
    #[error("Command execution caused an error.")]
    IoError(#[from] io::Error),
    #[error("Command resulted in error code: {0}.")]
    CommandResult(i32),
    #[error("Platform not supported by notifier. Contact logansaso+tech@gmail.com with your details for support.")]
    PlatformNotSupported,
    //region Config
    #[error("Error loading or saving configuration")]
    Config(#[from] serde_json::Error),
    //endregion
    #[error("Product is not in stock")]
    NoScrapingTargetFound,
    #[error("Rate Limit")]
    RateLimit,
    #[error("Web request failed.")]
    WebRequestFailed(#[from] reqwest::Error),
    #[error("Bad Status in Response")]
    BadStatus(reqwest::StatusCode),
    #[error("Bad Status from WebServer")]
    WebServer(reqwest::StatusCode),
    #[error("Bad Status from WebClient")]
    WebClient(reqwest::StatusCode),
    #[error("Proxy Not Running")]
    ProxyNotRunning,
    #[error("Building the Web Client Failed")]
    ClientBuild(reqwest::Error),
}
