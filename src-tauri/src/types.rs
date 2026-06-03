use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ColumnMapping {
    pub mjs_header: String,
    pub other_header: String,
    pub is_compare: bool,
    pub tolerance: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonConfig {
    pub mjs_path: String,
    pub mjs_encoding: String,
    pub other_path: String,
    pub other_encoding: String,
    pub output_dir: String,
    pub columns: Vec<ColumnMapping>,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryRow {
    pub item_name: String,
    pub result: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonResult {
    pub summary: Vec<SummaryRow>,
    pub summary_file: String,
    pub detail_file: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfigData {
    pub columns: Vec<ColumnMapping>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProgressPayload {
    pub message: String,
    pub step: u32,
    pub total: u32,
}
