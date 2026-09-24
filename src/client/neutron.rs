use anyhow::Result;
use serde::Deserialize;
use crate::client::OpenStackClient;

// ── Network types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct NetworkList {
    pub networks: Vec<Network>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Network {
    pub id: String,
    pub name: String,
    pub admin_state_up: bool,
    pub status: String,
    #[serde(rename = "provider:network_type")]
    pub network_type: Option<String>,   // flat | vlan | vxlan | gre
    #[serde(rename = "provider:physical_network")]
    pub physical_network: Option<String>,
    #[serde(rename = "provider:segmentation_id")]
    pub segmentation_id: Option<u32>,
    pub shared: bool,
    pub external: Option<bool>,
    pub subnets: Option<Vec<String>>,
}

// ── Agent types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct AgentList {
    pub agents: Vec<Agent>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Agent {
    pub id: String,
    pub agent_type: String,           // "Open vSwitch agent", "L3 agent", …
    pub binary: String,
    pub host: String,
    pub alive: bool,
    pub admin_state_up: bool,
    pub description: Option<String>,
    pub heartbeat_timestamp: Option<String>,
    pub started_at: Option<String>,
}

// ── Router types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct RouterList {
    pub routers: Vec<Router>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Router {
    pub id: String,
    pub name: String,
    pub status: String,
    pub admin_state_up: bool,
    pub external_gateway_info: Option<serde_json::Value>,
    pub distributed: Option<bool>,
    pub ha: Option<bool>,
}

// ── API functions ─────────────────────────────────────────────────────────────

/// GET /v2.0/networks
pub async fn list_networks(client: &OpenStackClient) -> Result<Vec<Network>> {
    let base = client.endpoint("network")?;
    let url = format!("{}/v2.0/networks", base);
    tracing::debug!("GET {}", url);
    let resp: NetworkList = client.get(&url).send().await?.error_for_status()?.json().await?;
    Ok(resp.networks)
}

/// GET /v2.0/agents
pub async fn list_agents(client: &OpenStackClient) -> Result<Vec<Agent>> {
    let base = client.endpoint("network")?;
    let url = format!("{}/v2.0/agents", base);
    tracing::debug!("GET {}", url);
    let resp: AgentList = client.get(&url).send().await?.error_for_status()?.json().await?;
    Ok(resp.agents)
}

/// GET /v2.0/routers
pub async fn list_routers(client: &OpenStackClient) -> Result<Vec<Router>> {
    let base = client.endpoint("network")?;
    let url = format!("{}/v2.0/routers", base);
    tracing::debug!("GET {}", url);
    let resp: RouterList = client.get(&url).send().await?.error_for_status()?.json().await?;
    Ok(resp.routers)
}
