// src-tauri/src/ipc_commands.rs
// Tauri command handlers exposing R Daemon and Form Registry to the Svelte frontend

use std::sync::Arc;
use serde_json::Value;
use tauri::State;

use crate::data_bridge::{AnalysisResult, ComputeResult, DataSlice, DatasetMetadata, OutputBlock, VariableMeta};
use crate::form_registry::{FormRegistry, MenuNode};
use crate::r_daemon::RDaemon;

pub struct AppState {
    pub r_daemon: Arc<RDaemon>,
    pub form_registry: Arc<FormRegistry>,
}

#[tauri::command]
pub async fn ping_r(state: State<'_, AppState>) -> Result<Value, String> {
    state.r_daemon.send_request("PING", serde_json::json!({})).await
}

#[tauri::command]
pub async fn load_dataset(state: State<'_, AppState>, file_path: String) -> Result<DatasetMetadata, String> {
    let res = state
        .r_daemon
        .send_request("LOAD_DATASET", serde_json::json!({ "file_path": file_path }))
        .await?;

    let meta: DatasetMetadata = serde_json::from_value(res)
        .map_err(|e| format!("Failed to parse dataset metadata: {e}"))?;
    Ok(meta)
}

#[tauri::command]
pub async fn get_dataset_metadata(state: State<'_, AppState>) -> Result<DatasetMetadata, String> {
    let res = state
        .r_daemon
        .send_request("GET_METADATA", serde_json::json!({}))
        .await?;

    let meta: DatasetMetadata = serde_json::from_value(res)
        .map_err(|e| format!("Failed to parse dataset metadata: {e}"))?;
    Ok(meta)
}

#[tauri::command]
pub async fn update_variable_metadata(
    state: State<'_, AppState>,
    col_name: String,
    updates: Value,
) -> Result<Vec<VariableMeta>, String> {
    let res = state
        .r_daemon
        .send_request(
            "UPDATE_VARIABLE",
            serde_json::json!({ "col_name": col_name, "updates": updates }),
        )
        .await?;

    let vars: Vec<VariableMeta> = serde_json::from_value(res["variables"].clone())
        .map_err(|e| format!("Failed to parse updated variables: {e}"))?;
    Ok(vars)
}

#[tauri::command]
pub async fn get_data_page(
    state: State<'_, AppState>,
    offset: usize,
    limit: usize,
) -> Result<DataSlice, String> {
    let res = state
        .r_daemon
        .send_request(
            "GET_DATA_PAGE",
            serde_json::json!({ "offset": offset, "limit": limit }),
        )
        .await?;

    let slice: DataSlice = serde_json::from_value(res)
        .map_err(|e| format!("Failed to parse data page: {e}"))?;
    Ok(slice)
}

#[tauri::command]
pub async fn execute_analysis(
    state: State<'_, AppState>,
    r_code: String,
) -> Result<AnalysisResult, String> {
    let res = state.r_daemon.execute_analysis(&r_code).await?;
    let result: AnalysisResult = serde_json::from_value(res)
        .map_err(|e| format!("Failed to parse analysis result: {e}"))?;
    Ok(result)
}

#[tauri::command]
pub async fn cancel_analysis(state: State<'_, AppState>) -> Result<(), String> {
    state.r_daemon.cancel().await
}

#[tauri::command]
pub async fn get_forms_menu(state: State<'_, AppState>) -> Result<Vec<MenuNode>, String> {
    Ok(state.form_registry.get_menu().await)
}

#[tauri::command]
pub async fn get_form_by_id(state: State<'_, AppState>, id: String) -> Result<Value, String> {
    state
        .form_registry
        .get_form(&id)
        .await
        .ok_or_else(|| format!("Form with id '{id}' not found"))
}

#[tauri::command]
pub async fn save_custom_form(state: State<'_, AppState>, form_json: Value) -> Result<String, String> {
    state.form_registry.save_custom_form(form_json).await
}

#[tauri::command]
pub async fn rename_variable(
    state: State<'_, AppState>,
    old_name: String,
    new_name: String,
) -> Result<Vec<VariableMeta>, String> {
    let res = state
        .r_daemon
        .send_request(
            "RENAME_VARIABLE",
            serde_json::json!({ "old_name": old_name, "new_name": new_name }),
        )
        .await?;

    let vars: Vec<VariableMeta> = serde_json::from_value(res["variables"].clone())
        .map_err(|e| format!("Failed to parse renamed variables: {e}"))?;
    Ok(vars)
}

#[tauri::command]
pub async fn add_variable(
    state: State<'_, AppState>,
    name: Option<String>,
    measure: Option<String>,
    label: Option<String>,
) -> Result<DatasetMetadata, String> {
    let res = state
        .r_daemon
        .send_request(
            "ADD_VARIABLE",
            serde_json::json!({ "name": name, "measure": measure, "label": label }),
        )
        .await?;

    let meta: DatasetMetadata = serde_json::from_value(res)
        .map_err(|e| format!("Failed to parse dataset metadata after adding variable: {e}"))?;
    Ok(meta)
}

#[tauri::command]
pub async fn compute_variable(
    state: State<'_, AppState>,
    target_var: String,
    expression: String,
) -> Result<ComputeResult, String> {
    let res = state
        .r_daemon
        .send_request(
            "COMPUTE_VARIABLE",
            serde_json::json!({ "target_var": target_var, "expression": expression }),
        )
        .await?;

    let dataset: DatasetMetadata = serde_json::from_value(res.clone())
        .map_err(|e| format!("Failed to parse computed dataset metadata: {e}"))?;
    let blocks: Vec<OutputBlock> = serde_json::from_value(res["blocks"].clone())
        .unwrap_or_default();

    Ok(ComputeResult { dataset, blocks })
}

#[tauri::command]
pub async fn export_dataset(
    state: State<'_, AppState>,
    file_path: String,
    format: String,
) -> Result<Value, String> {
    state
        .r_daemon
        .send_request(
            "EXPORT_DATASET",
            serde_json::json!({ "file_path": file_path, "format": format }),
        )
        .await
}

#[tauri::command]
pub async fn save_project(file_path: String, project_data: Value) -> Result<(), String> {
    let json_str = serde_json::to_string_pretty(&project_data)
        .map_err(|e| format!("Failed to serialize project: {e}"))?;
    tokio::fs::write(&file_path, json_str)
        .await
        .map_err(|e| format!("Failed to save project file: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn open_project(state: State<'_, AppState>, file_path: String) -> Result<Value, String> {
    let content = tokio::fs::read_to_string(&file_path)
        .await
        .map_err(|e| format!("Failed to read project file: {e}"))?;
    let project_data: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid project JSON: {e}"))?;

    // If project contains dataset rows and variables, restore directly into active R session
    if let Some(ds) = project_data.get("dataset") {
        let rows = ds.get("rows").cloned().unwrap_or(serde_json::json!([]));
        let vars = ds.get("variables").cloned().unwrap_or(serde_json::json!([]));
        let ds_name = ds.get("dataset_name").and_then(|n| n.as_str()).unwrap_or("Project Dataset");

        let _ = state
            .r_daemon
            .send_request(
                "LOAD_RAW_DATASET",
                serde_json::json!({
                    "rows": rows,
                    "variables": vars,
                    "dataset_name": ds_name
                }),
            )
            .await;
    }

    Ok(project_data)
}

#[tauri::command]
pub async fn pick_file(
    title: Option<String>,
    filter_name: Option<String>,
    extensions: Vec<String>,
) -> Result<Option<String>, String> {
    let mut dialog = rfd::AsyncFileDialog::new();
    if let Some(t) = title {
        dialog = dialog.set_title(&t);
    }
    if !extensions.is_empty() {
        let exts: Vec<&str> = extensions.iter().map(|s| s.as_str()).collect();
        let fname = filter_name.as_deref().unwrap_or("Supported Files");
        dialog = dialog.add_filter(fname, &exts);
    }
    let res = dialog.pick_file().await;
    Ok(res.map(|f| f.path().to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn save_file(
    title: Option<String>,
    default_name: Option<String>,
    filter_name: Option<String>,
    extensions: Vec<String>,
) -> Result<Option<String>, String> {
    let mut dialog = rfd::AsyncFileDialog::new();
    if let Some(t) = title {
        dialog = dialog.set_title(&t);
    }
    if let Some(d) = default_name {
        dialog = dialog.set_file_name(&d);
    }
    if !extensions.is_empty() {
        let exts: Vec<&str> = extensions.iter().map(|s| s.as_str()).collect();
        let fname = filter_name.as_deref().unwrap_or("Supported Files");
        dialog = dialog.add_filter(fname, &exts);
    }
    let res = dialog.save_file().await;
    Ok(res.map(|f| f.path().to_string_lossy().to_string()))
}
