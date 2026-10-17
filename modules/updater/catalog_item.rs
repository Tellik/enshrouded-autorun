use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogItem {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub summary: String,
    pub icon_url: Option<String>,
    pub author: Option<String>,
    pub downloads: Option<u64>,
    pub version: Option<String>,
}
