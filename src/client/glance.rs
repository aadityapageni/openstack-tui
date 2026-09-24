use anyhow::Result;
use serde::Deserialize;
use crate::client::OpenStackClient;

#[derive(Debug, Clone, Deserialize)]
pub struct ImageList {
    pub images: Vec<Image>,
    pub first: Option<String>,
    pub next: Option<String>,
    pub schema: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Image {
    pub id: String,
    pub name: Option<String>,
    pub status: String,           // active | queued | saving | deactivated
    pub visibility: Option<String>, // public | private | shared | community
    pub disk_format: Option<String>,
    pub container_format: Option<String>,
    pub size: Option<u64>,
    pub virtual_size: Option<u64>,
    pub min_disk: Option<u32>,
    pub min_ram: Option<u32>,
    pub protected: Option<bool>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub owner: Option<String>,
}

/// GET /v2/images
pub async fn list_images(client: &OpenStackClient) -> Result<Vec<Image>> {
    let base = client.endpoint("image")?;
    let url = format!("{}/v2/images?limit=200", base);
    tracing::debug!("GET {}", url);
    let resp: ImageList = client.get(&url).send().await?.error_for_status()?.json().await?;
    Ok(resp.images)
}
