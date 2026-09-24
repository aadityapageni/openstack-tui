use anyhow::Result;
use serde::Deserialize;
use crate::client::OpenStackClient;

// ── Hypervisor types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct HypervisorList {
    pub hypervisors: Vec<Hypervisor>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Hypervisor {
    pub id: String,
    pub hypervisor_hostname: String,
    pub state: String,           // "up" | "down"
    pub status: String,          // "enabled" | "disabled"
    pub vcpus: u32,
    pub vcpus_used: u32,
    pub memory_mb: u64,
    pub memory_mb_used: u64,
    pub local_gb: u64,
    pub local_gb_used: u64,
    pub running_vms: u32,
    pub hypervisor_type: Option<String>,
    pub hypervisor_version: Option<u64>,
}

// ── Server (VM) types ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct ServerList {
    pub servers: Vec<Server>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    pub id: String,
    pub name: String,
    pub status: String,          // ACTIVE | SHUTOFF | ERROR | BUILD | …
    #[serde(rename = "OS-EXT-STS:power_state")]
    pub power_state: Option<u8>,
    #[serde(rename = "OS-EXT-AZ:availability_zone")]
    pub availability_zone: Option<String>,
    #[serde(rename = "OS-EXT-SRV-ATTR:host")]
    pub host: Option<String>,
    #[serde(rename = "OS-EXT-SRV-ATTR:hypervisor_hostname")]
    pub hypervisor_hostname: Option<String>,
    pub tenant_id: Option<String>,
    pub user_id: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub flavor: Option<FlavorRef>,
    pub image: Option<serde_json::Value>,
    pub addresses: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FlavorRef {
    pub id: Option<String>,
    // Nova microversion 2.47+ embeds the flavor inline
    pub vcpus: Option<u32>,
    pub ram: Option<u64>,
    pub disk: Option<u64>,
    pub original_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImageRef {
    pub id: Option<String>,
    pub name: Option<String>,
}

// ── Nova service health ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct NovaServiceList {
    pub services: Vec<NovaService>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NovaService {
    pub binary: String,      // "nova-compute", "nova-scheduler", …
    pub host: String,
    pub zone: String,
    pub status: String,      // "enabled" | "disabled"
    pub state: String,       // "up" | "down"
    pub updated_at: Option<String>,
    pub disabled_reason: Option<String>,
}

// ── API functions ─────────────────────────────────────────────────────────────

/// GET /v2.1/os-hypervisors/detail
pub async fn list_hypervisors(client: &OpenStackClient) -> Result<Vec<Hypervisor>> {
    let base = client.endpoint("compute")?;
    let url = format!("{}/os-hypervisors/detail", base);

    tracing::debug!("GET {}", url);

    let resp: HypervisorList = client
        .get(&url)
        .header("X-OpenStack-Nova-API-Version", "2.79")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(resp.hypervisors)
}

/// GET /v2.1/servers/detail?all_tenants=1
pub async fn list_servers(client: &OpenStackClient) -> Result<Vec<Server>> {
    let base = client.endpoint("compute")?;
    let url = format!("{}/servers/detail?all_tenants=1&limit=500", base);

    tracing::debug!("GET {}", url);

    let resp: ServerList = client
        .get(&url)
        .header("X-OpenStack-Nova-API-Version", "2.79")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(resp.servers)
}

/// GET /v2.1/os-services
pub async fn list_nova_services(client: &OpenStackClient) -> Result<Vec<NovaService>> {
    let base = client.endpoint("compute")?;
    let url = format!("{}/os-services", base);

    tracing::debug!("GET {}", url);

    let resp: NovaServiceList = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(resp.services)
}
