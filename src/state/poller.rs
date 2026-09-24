use std::time::Duration;

use chrono::Utc;
use tokio::time;

use crate::auth::keystone::reauthenticate;
use crate::client::OpenStackClient;
use crate::state::SharedState;

/// Spawn all background polling tasks.
/// Each task runs on the Tokio runtime and updates SharedState independently.
pub async fn spawn_pollers(state: SharedState) {
    let cfg = {
        let s = state.lock().await;
        s.cloud_config.clone()
    };

    // ── Nova poller ────────────────────────────────────────────────────────
    tokio::spawn(nova_poller(state.clone(), cfg.clone(), Duration::from_secs(10)));

    // ── Neutron poller ─────────────────────────────────────────────────────
    tokio::spawn(neutron_poller(state.clone(), cfg.clone(), Duration::from_secs(15)));

    // ── Swift poller ───────────────────────────────────────────────────────
    tokio::spawn(swift_poller(state.clone(), cfg.clone(), Duration::from_secs(30)));

    // ── Cinder poller ──────────────────────────────────────────────────────
    tokio::spawn(cinder_poller(state.clone(), cfg.clone(), Duration::from_secs(20)));

    // ── Glance poller ──────────────────────────────────────────────────────
    tokio::spawn(glance_poller(state.clone(), cfg.clone(), Duration::from_secs(60)));

    // ── Token refresher ────────────────────────────────────────────────────
    tokio::spawn(token_refresher(state.clone(), cfg.clone()));
}

// ── Helper: build a client from current shared state ─────────────────────────

async fn make_client(state: &SharedState) -> anyhow::Result<OpenStackClient> {
    let s = state.lock().await;
    OpenStackClient::new(&s.cloud_config, &s.auth_token)
}

// ── Nova poller ───────────────────────────────────────────────────────────────

async fn nova_poller(
    state: SharedState,
    _cfg: crate::config::clouds::CloudConfig,
    interval: Duration,
) {
    let mut ticker = time::interval(interval);
    loop {
        ticker.tick().await;
        match make_client(&state).await {
            Ok(client) => {
                let hypervisors = crate::client::nova::list_hypervisors(&client).await;
                let servers = crate::client::nova::list_servers(&client).await;
                let nova_svcs = crate::client::nova::list_nova_services(&client).await;

                let mut s = state.lock().await;
                match (hypervisors, servers, nova_svcs) {
                    (Ok(h), Ok(srv), Ok(nsvc)) => {
                        s.hypervisors = h;
                        s.servers = srv;
                        s.nova_services = nsvc;
                        s.nova_status.last_updated = Some(Utc::now());
                        s.nova_status.error = None;
                    }
                    (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
                        tracing::warn!("Nova poller error: {:#}", e);
                        s.nova_status.error = Some(e.to_string());
                    }
                }
            }
            Err(e) => tracing::error!("Nova poller: failed to build client: {:#}", e),
        }
    }
}

// ── Neutron poller ────────────────────────────────────────────────────────────

async fn neutron_poller(
    state: SharedState,
    _cfg: crate::config::clouds::CloudConfig,
    interval: Duration,
) {
    let mut ticker = time::interval(interval);
    loop {
        ticker.tick().await;
        match make_client(&state).await {
            Ok(client) => {
                let networks = crate::client::neutron::list_networks(&client).await;
                let agents = crate::client::neutron::list_agents(&client).await;
                let routers = crate::client::neutron::list_routers(&client).await;

                let mut s = state.lock().await;
                match (networks, agents, routers) {
                    (Ok(n), Ok(a), Ok(r)) => {
                        s.networks = n;
                        s.agents = a;
                        s.routers = r;
                        s.neutron_status.last_updated = Some(Utc::now());
                        s.neutron_status.error = None;
                    }
                    (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
                        tracing::warn!("Neutron poller error: {:#}", e);
                        s.neutron_status.error = Some(e.to_string());
                    }
                }
            }
            Err(e) => tracing::error!("Neutron poller: failed to build client: {:#}", e),
        }
    }
}

// ── Swift poller ──────────────────────────────────────────────────────────────

async fn swift_poller(
    state: SharedState,
    _cfg: crate::config::clouds::CloudConfig,
    interval: Duration,
) {
    let mut ticker = time::interval(interval);
    loop {
        ticker.tick().await;
        match make_client(&state).await {
            Ok(client) => {
                let info = crate::client::swift::get_info(&client).await;
                // TODO: extract project_id from token and fetch account stats
                let mut s = state.lock().await;
                match info {
                    Ok(i) => {
                        s.swift_info = Some(i);
                        s.swift_status.last_updated = Some(Utc::now());
                        s.swift_status.error = None;
                    }
                    Err(e) => {
                        tracing::warn!("Swift poller error: {:#}", e);
                        s.swift_status.error = Some(e.to_string());
                    }
                }
            }
            Err(e) => tracing::error!("Swift poller: failed to build client: {:#}", e),
        }
    }
}

// ── Cinder poller ─────────────────────────────────────────────────────────────

async fn cinder_poller(
    state: SharedState,
    _cfg: crate::config::clouds::CloudConfig,
    interval: Duration,
) {
    let mut ticker = time::interval(interval);
    loop {
        ticker.tick().await;
        match make_client(&state).await {
            Ok(client) => {
                let volumes = crate::client::cinder::list_volumes(&client).await;
                let services = crate::client::cinder::list_cinder_services(&client).await;

                let mut s = state.lock().await;
                match (volumes, services) {
                    (Ok(v), Ok(sv)) => {
                        s.volumes = v;
                        s.cinder_services = sv;
                        s.cinder_status.last_updated = Some(Utc::now());
                        s.cinder_status.error = None;
                    }
                    (Err(e), _) | (_, Err(e)) => {
                        tracing::warn!("Cinder poller error: {:#}", e);
                        s.cinder_status.error = Some(e.to_string());
                    }
                }
            }
            Err(e) => tracing::error!("Cinder poller: failed to build client: {:#}", e),
        }
    }
}

// ── Glance poller ─────────────────────────────────────────────────────────────

async fn glance_poller(
    state: SharedState,
    _cfg: crate::config::clouds::CloudConfig,
    interval: Duration,
) {
    let mut ticker = time::interval(interval);
    loop {
        ticker.tick().await;
        match make_client(&state).await {
            Ok(client) => {
                match crate::client::glance::list_images(&client).await {
                    Ok(images) => {
                        let mut s = state.lock().await;
                        s.images = images;
                        s.glance_status.last_updated = Some(Utc::now());
                        s.glance_status.error = None;
                    }
                    Err(e) => {
                        tracing::warn!("Glance poller error: {:#}", e);
                        let mut s = state.lock().await;
                        s.glance_status.error = Some(e.to_string());
                    }
                }
            }
            Err(e) => tracing::error!("Glance poller: failed to build client: {:#}", e),
        }
    }
}

// ── Token refresher ────────────────────────────────────────────────────────────

/// Checks every 60 seconds if the token expires within 5 minutes.
/// If so, re-authenticates and updates the shared state.
async fn token_refresher(
    state: SharedState,
    cfg: crate::config::clouds::CloudConfig,
) {
    let mut ticker = time::interval(Duration::from_secs(60));
    loop {
        ticker.tick().await;
        let expires_soon = {
            let s = state.lock().await;
            s.auth_token.expires_soon(300) // refresh if < 5 min left
        };

        if expires_soon {
            tracing::info!("Token expiring soon, re-authenticating...");
            match reauthenticate(&cfg).await {
                Ok(new_token) => {
                    let mut s = state.lock().await;
                    s.auth_token = new_token;
                    tracing::info!("Token refreshed successfully");
                }
                Err(e) => tracing::error!("Token refresh failed: {:#}", e),
            }
        }
    }
}
