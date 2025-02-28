use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentDataResponse {
    pub success: bool,
    pub code: i64,
    pub result: Option<ProductResult>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductResult {
    pub uuid: String,
    pub create_time: i64,
    #[serde(rename = "created_at")]
    pub created_at: String,
    pub data_str: String,
    pub description: String,
    #[serde(rename = "display_title")]
    pub display_title: String,
    pub doc_type: i64,
    pub public: bool,
    pub source: String,
    pub ticket: i64,
    pub title: String,
    #[serde(rename = "type")]
    pub type_field: i64,
    pub update_time: i64,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    pub version: i64,
    #[serde(rename = "3d_model_uuid")]
    pub n3d_model_uuid: String,
    #[serde(rename = "std_uuid")]
    pub std_uuid: String,
    #[serde(rename = "has_device")]
    pub has_device: bool,
    pub path: String,
}