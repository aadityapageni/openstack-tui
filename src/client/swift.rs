use anyhow::Result;
use serde::Deserialize;
use crate::client::OpenStackClient;

/// Swift cluster info from /info endpoint
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SwiftInfo {
    pub swift: Option<SwiftCore>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SwiftCore {
    pub version: Option<String>,
    pub max_file_size: Option<u64>,
    pub max_meta_count: Option<u32>,
}

/// Swift account stats (from HEAD /v1/AUTH_<project_id>)
#[derive(Debug, Clone, Default)]
pub struct SwiftAccountStats {
    pub container_count: u64,
    pub object_count: u64,
    pub bytes_used: u64,
}

/// GET /info
pub async fn get_info(client: &OpenStackClient) -> Result<SwiftInfo> {
    let base = client.endpoint("object-store")?;
    let url = format!("{}/info", base);
    tracing::debug!("GET {}", url);
    let info: SwiftInfo = client.get(&url).send().await?.error_for_status()?.json().await?;
    Ok(info)
}

/// HEAD /v1/AUTH_<project> → read account stats from response headers
pub async fn get_account_stats(
    client: &OpenStackClient,
    project_id: &str,
) -> Result<SwiftAccountStats> {
    let base = client.endpoint("object-store")?;
    let url = format!("{}/v1/AUTH_{}", base, project_id);
    tracing::debug!("HEAD {}", url);

    let resp = client
        .inner
        .head(&url)
        .header("X-Auth-Token", &client.token)
        .send()
        .await?
        .error_for_status()?;

    let headers = resp.headers();
    let parse_header = |key: &str| -> u64 {
        headers
            .get(key)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    };

    Ok(SwiftAccountStats {
        container_count: parse_header("X-Account-Container-Count"),
        object_count: parse_header("X-Account-Object-Count"),
        bytes_used: parse_header("X-Account-Bytes-Used"),
    })
}
