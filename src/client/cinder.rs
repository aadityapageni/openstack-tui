use anyhow::Result;
use serde::Deserialize;
use crate::client::OpenStackClient;

// ── Volume types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct VolumeList {
    pub volumes: Vec<Volume>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Volume {
    pub id: String,
    pub name: Option<String>,
    pub status: String,          // available | in-use | error | deleting …
    pub size: u64,               // GiB
    pub volume_type: Option<String>,
    pub bootable: Option<String>,
    pub encrypted: Option<bool>,
    pub attachments: Option<Vec<VolumeAttachment>>,
    pub created_at: Option<String>,
    pub host: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VolumeAttachment {
    pub server_id: Option<String>,
    pub device: Option<String>,
}

// ── Cinder service types ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct CinderServiceList {
    pub services: Vec<CinderService>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CinderService {
    pub binary: String,        // "cinder-volume" | "cinder-scheduler"
    pub host: String,
    pub zone: String,
    pub status: String,        // "enabled" | "disabled"
    pub state: String,         // "up" | "down"
    pub updated_at: Option<String>,
    pub disabled_reason: Option<String>,
}

// ── API functions ─────────────────────────────────────────────────────────────

/// GET /v3/volumes/detail?all_tenants=1
pub async fn list_volumes(client: &OpenStackClient) -> Result<Vec<Volume>> {
    let base = client.endpoint("block-storage").or_else(|_| client.endpoint("volumev3"))?;
    let url = format!("{}/volumes/detail?all_tenants=1&limit=200", base);
    tracing::debug!("GET {}", url);
    let resp: VolumeList = client.get(&url).send().await?.error_for_status()?.json().await?;
    Ok(resp.volumes)
}

/// GET /v3/os-services
pub async fn list_cinder_services(client: &OpenStackClient) -> Result<Vec<CinderService>> {
    let base = client.endpoint("block-storage").or_else(|_| client.endpoint("volumev3"))?;
    let url = format!("{}/os-services", base);
    tracing::debug!("GET {}", url);
    let resp: CinderServiceList = client.get(&url).send().await?.error_for_status()?.json().await?;
    Ok(resp.services)
}
