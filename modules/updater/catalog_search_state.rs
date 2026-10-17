use crate::catalog_item::CatalogItem;
use serde::Serialize;

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogState {
    pub state: String,
    pub query: String,
    pub message: Option<String>,
    pub items: Vec<CatalogItem>,
}
