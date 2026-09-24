use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::config::clouds::CloudConfig;

/// A resolved Keystone v3 token with service catalog
#[derive(Debug, Clone)]
pub struct AuthToken {
    /// The X-Auth-Token header value
    pub token: String,
    /// When this token expires
    pub expires_at: DateTime<Utc>,
    /// Service catalog: service_type → interface → url
    pub catalog: ServiceCatalog,
}

pub type ServiceCatalog = HashMap<String, HashMap<String, String>>;

impl AuthToken {
    /// Returns true if the token expires within `threshold_secs` seconds
    pub fn expires_soon(&self, threshold_secs: i64) -> bool {
        let remaining = (self.expires_at - Utc::now()).num_seconds();
        remaining < threshold_secs
    }

    /// Remaining lifetime in seconds
    pub fn remaining_secs(&self) -> i64 {
        (self.expires_at - Utc::now()).num_seconds().max(0)
    }

    /// Look up an endpoint URL for a given service type and interface
    pub fn endpoint(&self, service_type: &str, interface: &str) -> Option<&str> {
        self.catalog
            .get(service_type)?
            .get(interface)
            .map(String::as_str)
    }
}

// ── Keystone v3 auth request/response types ───────────────────────────────

#[derive(Serialize)]
struct AuthRequest {
    auth: AuthBody,
}

#[derive(Serialize)]
struct AuthBody {
    identity: Identity,
    scope: Scope,
}

#[derive(Serialize)]
struct Identity {
    methods: Vec<String>,
    password: PasswordAuth,
}

#[derive(Serialize)]
struct PasswordAuth {
    user: UserRef,
}

#[derive(Serialize)]
struct UserRef {
    name: String,
    password: String,
    domain: DomainRef,
}

#[derive(Serialize)]
struct DomainRef {
    name: String,
}

#[derive(Serialize)]
struct Scope {
    project: ProjectRef,
}

#[derive(Serialize)]
struct ProjectRef {
    name: String,
    domain: DomainRef,
}

#[derive(Deserialize)]
struct TokenResponse {
    token: TokenBody,
}

#[derive(Deserialize)]
struct TokenBody {
    expires_at: DateTime<Utc>,
    catalog: Option<Vec<CatalogEntry>>,
}

#[derive(Deserialize)]
struct CatalogEntry {
    #[serde(rename = "type")]
    service_type: String,
    endpoints: Vec<Endpoint>,
}

#[derive(Deserialize)]
struct Endpoint {
    interface: String,
    url: String,
    region_id: Option<String>,
}

/// Authenticate against Keystone v3 and return a usable AuthToken.
pub async fn authenticate(config: &CloudConfig) -> Result<AuthToken> {
    let client = build_client(config)?;

    let auth_url = config.auth.auth_url.trim_end_matches('/');
    let url = format!("{}/v3/auth/tokens", auth_url);

    let body = AuthRequest {
        auth: AuthBody {
            identity: Identity {
                methods: vec!["password".to_string()],
                password: PasswordAuth {
                    user: UserRef {
                        name: config.auth.username.clone(),
                        password: config.auth.password.clone(),
                        domain: DomainRef {
                            name: config.auth.user_domain_name.clone(),
                        },
                    },
                },
            },
            scope: Scope {
                project: ProjectRef {
                    name: config.auth.project_name.clone(),
                    domain: DomainRef {
                        name: config.auth.project_domain_name.clone(),
                    },
                },
            },
        },
    };

    tracing::debug!("POST {}", url);

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .context("Failed to connect to Keystone")?;

    // Token is returned in X-Subject-Token header
    let token_str = response
        .headers()
        .get("X-Subject-Token")
        .context("Keystone response missing X-Subject-Token header")?
        .to_str()
        .context("Invalid token header value")?
        .to_string();

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Keystone auth failed ({}): {}", status, body);
    }

    let token_resp: TokenResponse = response
        .json()
        .await
        .context("Failed to parse Keystone token response")?;

    // Build service catalog: type → interface → url (filtered by region)
    let catalog = build_catalog(
        token_resp.token.catalog.unwrap_or_default(),
        &config.region_name,
    );

    tracing::info!(
        "Token acquired, expires at {}, {} services in catalog",
        token_resp.token.expires_at,
        catalog.len()
    );

    Ok(AuthToken {
        token: token_str,
        expires_at: token_resp.token.expires_at,
        catalog,
    })
}

/// Re-authenticate and return a fresh token (used by the token refresher task)
pub async fn reauthenticate(config: &CloudConfig) -> Result<AuthToken> {
    tracing::info!("Re-authenticating with Keystone");
    authenticate(config).await
}

fn build_client(config: &CloudConfig) -> Result<Client> {
    let verify = config.verify.unwrap_or(true);
    let mut builder = Client::builder()
        .danger_accept_invalid_certs(!verify)
        .timeout(std::time::Duration::from_secs(30));

    if let Some(cacert) = &config.cacert {
        let cert_pem = std::fs::read(cacert)
            .with_context(|| format!("Failed to read CA cert: {}", cacert))?;
        let cert = reqwest::Certificate::from_pem(&cert_pem)
            .context("Failed to parse CA certificate")?;
        builder = builder.add_root_certificate(cert);
    }

    builder.build().context("Failed to build HTTP client")
}

fn build_catalog(entries: Vec<CatalogEntry>, region: &str) -> ServiceCatalog {
    let mut catalog: ServiceCatalog = HashMap::new();

    for entry in entries {
        let iface_map = catalog.entry(entry.service_type.clone()).or_default();
        for ep in &entry.endpoints {
            // Prefer the matching region; fall back to any
            let is_right_region = ep
                .region_id
                .as_deref()
                .map(|r| r == region)
                .unwrap_or(true);
            if is_right_region {
                iface_map.insert(ep.interface.clone(), ep.url.clone());
            }
        }
    }

    catalog
}
