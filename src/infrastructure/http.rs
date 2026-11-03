use crate::core::errors::Result;
use reqwest::Client;
use std::time::Duration;

pub const DEFAULT_TIMEOUT_SECS: u64 = 10;
pub const USER_AGENT: &str = concat!("PackageDownloader/", env!("CARGO_PKG_VERSION"));

pub fn build_http_client(timeout_secs: Option<u64>) -> Result<Client> {
    let timeout = Duration::from_secs(timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS));
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .timeout(timeout)
        .connect_timeout(Duration::from_secs(5))
        .pool_max_idle_per_host(5)
        .build()?;
    Ok(client)
}
