pub mod nova;
pub mod neutron;
pub mod swift;
pub mod cinder;
pub mod glance;

use anyhow::Result;
use reqwest::Client;

use crate::auth::keystone::AuthToken;
use crate::config::clouds::CloudConfig;

/// Shared authenticated HTTP client used by all service clients
pub struct OpenStackClient {
    pub inner: Client,
    pub token: String,
    pub interface: String,
    pub region: String,
    pub catalog: crate::auth::keystone::ServiceCatalog,
}

impl OpenStackClient {
    pub fn new(config: &CloudConfig, auth: &AuthToken) -> Result<Self> {
        let verify = config.verify.unwrap_or(true);
        let mut builder = Client::builder()
            .danger_accept_invalid_certs(!verify)
            .timeout(std::time::Duration::from_secs(30));

        if let Some(cacert) = &config.cacert {
            let cert_pem = std::fs::read(cacert)?;
            let cert = reqwest::Certificate::from_pem(&cert_pem)?;
            builder = builder.add_root_certificate(cert);
        }

        Ok(Self {
            inner: builder.build()?,
            token: auth.token.clone(),
            interface: config.interface.clone(),
            region: config.region_name.clone(),
            catalog: auth.catalog.clone(),
        })
    }

    /// Resolve a service endpoint URL from the catalog
    pub fn endpoint(&self, service_type: &str) -> Result<String> {
        let iface = &self.interface;
        let url = self
            .catalog
            .get(service_type)
            .and_then(|m| m.get(iface).or_else(|| m.get("public")))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No '{}' endpoint found for service '{}' in catalog",
                    iface,
                    service_type
                )
            })?;
        Ok(url.trim_end_matches('/').to_string())
    }

    /// Build a GET request pre-filled with auth token header
    pub fn get(&self, url: &str) -> reqwest::RequestBuilder {
        self.inner
            .get(url)
            .header("X-Auth-Token", &self.token)
    }
}
