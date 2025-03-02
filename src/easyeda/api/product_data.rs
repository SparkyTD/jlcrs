use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductDataResponse {
    pub code: i64,
    pub msg: Value,
    pub result: ProductsResult,
    pub success: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductsResult {
    pub total: i64,
    pub param_list: Vec<ParamList>,
    pub product_list: Vec<ProductInfo>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParamList {
    pub parameter_name: String,
    pub parameter_value_list: Vec<String>,
    #[serde(default)]
    pub parameter_id_list: Vec<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductInfo {
    #[serde(rename = "ifRoHS")]
    pub if_ro_hs: bool,
    pub price: Vec<(i64, String, String)>,
    pub stock: i64,
    pub mpn: String,
    pub number: String,
    pub package: String,
    pub manufacturer: String,
    pub url: String,
    pub image: Vec<Image>,
    pub mfr_link: String,
    pub stock_number: i64,
    pub price_list: Vec<PriceEntry>,
    pub has_device: String,
    #[serde(rename = "JLCPCB Part Class")]
    pub jlcpcb_part_class: String,
    #[serde(rename = "device_info")]
    pub device_info: DeviceInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub sort: i64,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "900x900")]
    pub n900x900: String,
    #[serde(rename = "224x224")]
    pub n224x224: String,
    #[serde(rename = "96x96")]
    pub n96x96: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceEntry {
    pub price: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub uuid: String,
    pub attributes: HashMap<String, String>,
    pub create_time: i64,
    #[serde(rename = "created_at")]
    pub created_at: String,
    pub creator: UserInfo,
    pub description: String,
    #[serde(rename = "display_title")]
    pub display_title: String,
    #[serde(rename = "footprint_type")]
    pub footprint_type: i64,
    pub images: Vec<String>,
    pub modifier: UserInfo,
    pub owner: UserInfo,
    #[serde(rename = "product_code")]
    pub product_code: String,
    #[serde(rename = "project_uuid")]
    pub project_uuid: String,
    pub source: String,
    #[serde(rename = "symbol_type")]
    pub symbol_type: i64,
    pub ticket: i64,
    pub title: String,
    pub update_time: i64,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    #[serde(rename = "Description")]
    pub description2: String,
    #[serde(rename = "symbol_info")]
    pub symbol_info: SymbolInfo,
    #[serde(rename = "footprint_info")]
    pub footprint_info: FootprintInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub uuid: String,
    pub username: Option<String>,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolInfo {
    pub uuid: String,
    pub create_time: i64,
    #[serde(rename = "created_at")]
    pub created_at: String,
    pub creator: UserInfo,
    pub data_str: String,
    pub description: String,
    #[serde(rename = "display_title")]
    pub display_title: String,
    pub doc_type: i64,
    pub modifier: UserInfo,
    pub owner: UserInfo,
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
    #[serde(rename = "std_uuid")]
    pub std_uuid: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FootprintInfo {
    pub uuid: String,
    pub create_time: i64,
    #[serde(rename = "created_at")]
    pub created_at: String,
    pub creator: UserInfo,
    pub data_str: String,
    pub description: String,
    #[serde(rename = "display_title")]
    pub display_title: String,
    pub doc_type: i64,
    pub modifier: UserInfo,
    pub owner: UserInfo,
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
    #[serde(rename = "std_uuid")]
    pub std_uuid: String,
    #[serde(rename = "model_3d")]
    pub model_3d: Option<Model3d>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Model3d {
    pub title: String,
    pub uri: String,
    pub transform: String,
}
