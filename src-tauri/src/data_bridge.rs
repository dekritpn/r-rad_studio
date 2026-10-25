// src-tauri/src/data_bridge.rs
// Serialization models for R Daemon IPC and Frontend Communication

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueLabel {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableMeta {
    pub name: String,
    #[serde(rename = "type")]
    pub var_type: String,
    pub width: u32,
    pub decimals: u32,
    pub label: String,
    #[serde(default)]
    pub values: Vec<ValueLabel>,
    #[serde(default = "default_missing")]
    pub missing: String,
    pub columns: u32,
    pub align: String,
    pub measure: String, // "scale" | "ordinal" | "nominal"
    #[serde(default)]
    pub storage_type: Option<String>,
}

fn default_missing() -> String {
    "None".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    pub dataset_name: String,
    pub row_count: usize,
    pub column_count: usize,
    pub variables: Vec<VariableMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputBlock {
    #[serde(rename = "type")]
    pub block_type: String, // "syntax" | "html_table" | "svg_plot" | "warning" | "error" | "text"
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub html: Option<String>,
    #[serde(default)]
    pub svg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub request_id: String,
    pub status: String,
    #[serde(default)]
    pub blocks: Vec<OutputBlock>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeResult {
    pub dataset: DatasetMetadata,
    #[serde(default)]
    pub blocks: Vec<OutputBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSlice {
    pub rows: Vec<Value>,
    pub total_rows: usize,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindReplaceResult {
    pub replaced_count: usize,
    pub dataset_name: String,
    pub row_count: usize,
    pub column_count: usize,
    pub variables: Vec<VariableMeta>,
}
