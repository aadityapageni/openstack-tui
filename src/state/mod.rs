use std::sync::Arc;
use tokio::sync::Mutex;

use crate::auth::keystone::AuthToken;
use crate::client::nova::{Hypervisor, NovaService, Server};
use crate::client::neutron::{Agent, Network, Router};
use crate::client::swift::{SwiftAccountStats, SwiftInfo};
use crate::client::cinder::{CinderService, Volume};
use crate::client::glance::Image;
use crate::config::clouds::CloudConfig;

pub mod poller;

/// Which view is currently active in the TUI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveView {
    Overview,
    Nodes,
    Servers,
    Networks,
    Swift,
    Volumes,
    Images,
    Services,
}

impl Default for ActiveView {
    fn default() -> Self {
        Self::Overview
    }
}

/// Last-fetch timestamps and error states per service
#[derive(Debug, Clone, Default)]
pub struct ServiceStatus {
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
    pub error: Option<String>,
}

/// The entire shared application state — updated by background pollers,
/// read by the TUI render thread via Arc<Mutex<AppState>>.
#[derive(Debug)]
pub struct AppState {
    // ── Identity ──────────────────────────────────────────────────────────
    pub cloud_name: String,
    pub cloud_config: CloudConfig,
    pub auth_token: AuthToken,

    // ── Compute (Nova) ────────────────────────────────────────────────────
    pub hypervisors: Vec<Hypervisor>,
    pub servers: Vec<Server>,
    pub nova_services: Vec<NovaService>,
    pub nova_status: ServiceStatus,

    // ── Network (Neutron) ─────────────────────────────────────────────────
    pub networks: Vec<Network>,
    pub agents: Vec<Agent>,
    pub routers: Vec<Router>,
    pub neutron_status: ServiceStatus,

    // ── Object Storage (Swift) ────────────────────────────────────────────
    pub swift_info: Option<SwiftInfo>,
    pub swift_stats: Option<SwiftAccountStats>,
    pub swift_status: ServiceStatus,

    // ── Block Storage (Cinder) ────────────────────────────────────────────
    pub volumes: Vec<Volume>,
    pub cinder_services: Vec<CinderService>,
    pub cinder_status: ServiceStatus,

    // ── Images (Glance) ───────────────────────────────────────────────────
    pub images: Vec<Image>,
    pub glance_status: ServiceStatus,

    // ── TUI navigation state ──────────────────────────────────────────────
    pub active_view: ActiveView,
    pub selected_index: usize,
    pub search_query: String,
    pub search_active: bool,
    pub detail_scroll: u16,
}

impl AppState {
    pub fn new(cloud_name: String, cloud_config: CloudConfig, auth_token: AuthToken) -> Self {
        Self {
            cloud_name,
            cloud_config,
            auth_token,
            hypervisors: vec![],
            servers: vec![],
            nova_services: vec![],
            nova_status: ServiceStatus::default(),
            networks: vec![],
            agents: vec![],
            routers: vec![],
            neutron_status: ServiceStatus::default(),
            swift_info: None,
            swift_stats: None,
            swift_status: ServiceStatus::default(),
            volumes: vec![],
            cinder_services: vec![],
            cinder_status: ServiceStatus::default(),
            images: vec![],
            glance_status: ServiceStatus::default(),
            active_view: ActiveView::default(),
            selected_index: 0,
            search_query: String::new(),
            search_active: false,
            detail_scroll: 0,
        }
    }

    /// Convenience: total vCPUs across all hypervisors
    pub fn total_vcpus(&self) -> u32 {
        self.hypervisors.iter().map(|h| h.vcpus).sum()
    }

    pub fn used_vcpus(&self) -> u32 {
        self.hypervisors.iter().map(|h| h.vcpus_used).sum()
    }

    pub fn total_memory_mb(&self) -> u64 {
        self.hypervisors.iter().map(|h| h.memory_mb).sum()
    }

    pub fn used_memory_mb(&self) -> u64 {
        self.hypervisors.iter().map(|h| h.memory_mb_used).sum()
    }

    pub fn nodes_up(&self) -> usize {
        self.hypervisors.iter().filter(|h| h.state == "up").count()
    }

    pub fn nodes_down(&self) -> usize {
        self.hypervisors.iter().filter(|h| h.state != "up").count()
    }

    pub fn active_servers(&self) -> usize {
        self.servers.iter().filter(|s| s.status == "ACTIVE").count()
    }

    pub fn agents_alive(&self) -> usize {
        self.agents.iter().filter(|a| a.alive).count()
    }

    pub fn agents_dead(&self) -> usize {
        self.agents.iter().filter(|a| !a.alive).count()
    }

    /// Token remaining time formatted for status bar
    pub fn token_remaining_display(&self) -> String {
        let secs = self.auth_token.remaining_secs();
        if secs > 3600 {
            format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
        } else if secs > 60 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else {
            format!("{}s ⚠️", secs)
        }
    }
}

pub type SharedState = Arc<Mutex<AppState>>;
