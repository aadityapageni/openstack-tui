use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Root structure of clouds.yaml
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CloudsFile {
    pub clouds: HashMap<String, CloudConfig>,
}

/// Per-cloud configuration block
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CloudConfig {
    pub auth: AuthConfig,
    /// Accept both `region_name: RegionOne` (scalar) and `regions: [RegionOne, RegionTwo]` (list).
    /// We always use the first one.
    #[serde(deserialize_with = "deserialize_region", default = "default_region")]
    pub region_name: String,
    #[serde(default = "default_interface")]
    pub interface: String,
    #[serde(default = "default_identity_version")]
    pub identity_api_version: u8,
    /// Skip TLS verification (false by default — do NOT disable in prod)
    #[serde(default)]
    pub verify: Option<bool>,
    /// Path to CA bundle for TLS verification
    pub cacert: Option<String>,
}

/// Deserializes either a scalar string or a list of strings,
/// always returning the first element.
fn deserialize_region<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum RegionField {
        Single(String),
        List(Vec<String>),
    }

    match RegionField::deserialize(deserializer)? {
        RegionField::Single(s) => Ok(s),
        RegionField::List(v) => Ok(v.into_iter().next().unwrap_or_else(default_region)),
    }
}

/// Auth sub-block inside a cloud entry
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    pub auth_url: String,
    pub username: String,
    pub password: String,
    #[serde(default = "default_project")]
    pub project_name: String,
    pub project_id: Option<String>,
    #[serde(default = "default_domain")]
    pub user_domain_name: String,
    #[serde(default = "default_domain")]
    pub project_domain_name: String,
}

fn default_region() -> String {
    "RegionOne".to_string()
}

fn default_interface() -> String {
    "public".to_string()
}

fn default_identity_version() -> u8 {
    3
}

fn default_domain() -> String {
    "Default".to_string()
}

fn default_project() -> String {
    "admin".to_string()
}

/// Load and parse a clouds.yaml file.
///
/// Resolution order:
/// 1. Explicit `path` argument (from --clouds CLI flag)
/// 2. `OS_CLIENT_CONFIG_FILE` environment variable
/// 3. `~/.config/openstack/clouds.yaml`
/// 4. `/etc/openstack/clouds.yaml`
pub fn load_clouds_yaml(path: Option<&str>) -> Result<CloudsFile> {
    let resolved = resolve_clouds_path(path)?;

    tracing::debug!("Loading clouds.yaml from: {}", resolved.display());

    let content = std::fs::read_to_string(&resolved)
        .with_context(|| format!("Failed to read clouds.yaml at {}", resolved.display()))?;

    let clouds: CloudsFile = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse clouds.yaml at {}", resolved.display()))?;

    tracing::info!(
        "Loaded {} cloud(s): {}",
        clouds.clouds.len(),
        clouds.clouds.keys().cloned().collect::<Vec<_>>().join(", ")
    );

    Ok(clouds)
}

fn resolve_clouds_path(explicit: Option<&str>) -> Result<PathBuf> {
    // 1. Explicit path
    if let Some(p) = explicit {
        return Ok(PathBuf::from(p));
    }

    // 2. Environment variable
    if let Ok(env_path) = std::env::var("OS_CLIENT_CONFIG_FILE") {
        return Ok(PathBuf::from(env_path));
    }

    // 3. XDG user config
    if let Some(config_dir) = dirs::config_dir() {
        let user_path = config_dir.join("openstack").join("clouds.yaml");
        if user_path.exists() {
            return Ok(user_path);
        }
    }

    // 4. System-wide
    let system_path = PathBuf::from("/etc/openstack/clouds.yaml");
    if system_path.exists() {
        return Ok(system_path);
    }

    anyhow::bail!(
        "clouds.yaml not found. Provide --clouds <path> or set OS_CLIENT_CONFIG_FILE, \
         or place it at ~/.config/openstack/clouds.yaml"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
clouds:
  admin:
    auth:
      auth_url: https://keystone.example.com:5000
      username: admin
      password: s3cr3t
      project_name: admin
      user_domain_name: Default
      project_domain_name: Default
    region_name: RegionOne
    interface: internal
    identity_api_version: 3
"#;

    const SAMPLE_REGIONS_LIST: &str = r#"
clouds:
  openstack:
    auth:
      auth_url: http://10.180.9.50:5000
      username: admin
      password: secret
      project_name: admin
      user_domain_name: Default
    regions:
    - RegionOne
    - RegionTwo
    interface: public
    identity_api_version: 3
"#;

    #[test]
    fn test_parse_sample() {
        let cf: CloudsFile = serde_yaml::from_str(SAMPLE).unwrap();
        let cloud = cf.clouds.get("admin").unwrap();
        assert_eq!(cloud.auth.username, "admin");
        assert_eq!(cloud.region_name, "RegionOne");
        assert_eq!(cloud.interface, "internal");
    }

    #[test]
    fn test_parse_regions_list() {
        let cf: CloudsFile = serde_yaml::from_str(SAMPLE_REGIONS_LIST).unwrap();
        let cloud = cf.clouds.get("openstack").unwrap();
        // Should take the first element of the list
        assert_eq!(cloud.region_name, "RegionOne");
        assert_eq!(cloud.interface, "public");
    }
}
