use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlInfo {
    pub url: String,
    pub site: String,
    pub title: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub streams: Value,
    pub caption: Caption,
    pub err: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamInfo {
    pub id: String,
    pub quality: String,
    pub parts: Vec<Parts>,
    pub size: i64,
    pub ext: String,
    #[serde(rename = "NeedMux")]
    pub need_mux: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parts {
    pub url: String,
    pub size: i64,
    pub ext: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Caption {
    pub danmaku: Danmaku,
    pub subtitle: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Danmaku {
    pub url: String,
    pub size: i64,
    pub ext: String,
}
