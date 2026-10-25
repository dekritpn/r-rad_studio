// src-tauri/tests/phase6_tests.rs
// Verification Gate 6: Multi-format I/O, Variable Renaming, Column Addition, Formula Computing, and Open Project Saving

use std::path::PathBuf;
use r_rad_studio_lib::data_bridge::{DatasetMetadata, OutputBlock};
use r_rad_studio_lib::r_daemon::{find_bridge_script, find_rscript_binary, RDaemon, RDaemonConfig};

fn get_test_daemon() -> RDaemon {
    let rscript = find_rscript_binary(None).expect("Failed to locate Rscript binary");
    let bridge = find_bridge_script();
    let config = RDaemonConfig {
        custom_rscript_path: Some(rscript),
        bridge_script_path: bridge,
    };
    RDaemon::new(config)
}

#[tokio::test]
async fn test_rename_and_add_variable() {
    let daemon = get_test_daemon();
    daemon.ensure_started().await.expect("Failed to start daemon");

    // Load mtcars
    daemon
        .send_request("LOAD_DATASET", serde_json::json!({ "file_path": "mtcars" }))
        .await
        .expect("Load mtcars failed");

    // Add variable
    let add_res = daemon
        .send_request(
            "ADD_VARIABLE",
            serde_json::json!({ "name": "NEW_SCORE", "measure": "scale", "label": "Test Score" }),
        )
        .await
        .expect("Add variable failed");

    let meta_after_add: DatasetMetadata = serde_json::from_value(add_res).unwrap();
    assert_eq!(meta_after_add.column_count, 12);
    assert!(meta_after_add.variables.iter().any(|v| v.name == "NEW_SCORE"));

    // Rename variable
    let rename_res = daemon
        .send_request(
            "RENAME_VARIABLE",
            serde_json::json!({ "old_name": "NEW_SCORE", "new_name": "RENAMED_SCORE" }),
        )
        .await
        .expect("Rename variable failed");

    let vars = rename_res["variables"].as_array().expect("Expected variables array");
    assert!(vars.iter().any(|v| v["name"] == "RENAMED_SCORE"));
    assert!(!vars.iter().any(|v| v["name"] == "NEW_SCORE"));
}

#[tokio::test]
async fn test_compute_variable() {
    let daemon = get_test_daemon();
    daemon.ensure_started().await.expect("Failed to start daemon");

    daemon
        .send_request("LOAD_DATASET", serde_json::json!({ "file_path": "mtcars" }))
        .await
        .expect("Load mtcars failed");

    // Compute new variable: log_hp = log(hp) * 10
    let comp_res = daemon
        .send_request(
            "COMPUTE_VARIABLE",
            serde_json::json!({ "target_var": "log_hp", "expression": "log(hp) * 10" }),
        )
        .await
        .expect("Compute variable failed");

    let meta: DatasetMetadata = serde_json::from_value(comp_res.clone()).unwrap();
    assert!(meta.variables.iter().any(|v| v.name == "log_hp"));

    let blocks: Vec<OutputBlock> = serde_json::from_value(comp_res["blocks"].clone()).unwrap();
    assert!(!blocks.is_empty(), "Expected output blocks from compute variable");
    assert!(blocks.iter().any(|b| b.block_type == "syntax"));
}

#[tokio::test]
async fn test_export_and_import_csv_sav_xlsx() {
    let daemon = get_test_daemon();
    daemon.ensure_started().await.expect("Failed to start daemon");

    daemon
        .send_request("LOAD_DATASET", serde_json::json!({ "file_path": "mtcars" }))
        .await
        .expect("Load mtcars failed");

    // 1. Export CSV and import back
    let csv_file = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
    let csv_path = csv_file.path().to_str().unwrap().to_string();
    daemon
        .send_request(
            "EXPORT_DATASET",
            serde_json::json!({ "file_path": csv_path, "format": "csv" }),
        )
        .await
        .expect("Export CSV failed");
    assert!(PathBuf::from(&csv_path).exists());

    let csv_load = daemon
        .send_request(
            "LOAD_DATASET",
            serde_json::json!({ "file_path": csv_path }),
        )
        .await
        .expect("Load exported CSV failed");
    let csv_meta: DatasetMetadata = serde_json::from_value(csv_load).unwrap();
    assert_eq!(csv_meta.row_count, 32);
    assert_eq!(csv_meta.column_count, 11);

    // 2. Export SAV and import back
    let sav_file = tempfile::Builder::new().suffix(".sav").tempfile().unwrap();
    let sav_path = sav_file.path().to_str().unwrap().to_string();
    daemon
        .send_request(
            "EXPORT_DATASET",
            serde_json::json!({ "file_path": sav_path, "format": "sav" }),
        )
        .await
        .expect("Export SAV failed");
    assert!(PathBuf::from(&sav_path).exists());

    let sav_load = daemon
        .send_request(
            "LOAD_DATASET",
            serde_json::json!({ "file_path": sav_path }),
        )
        .await
        .expect("Load exported SAV failed");
    let sav_meta: DatasetMetadata = serde_json::from_value(sav_load).unwrap();
    assert_eq!(sav_meta.row_count, 32);

    // 3. Export XLSX and import back
    let xlsx_file = tempfile::Builder::new().suffix(".xlsx").tempfile().unwrap();
    let xlsx_path = xlsx_file.path().to_str().unwrap().to_string();
    daemon
        .send_request(
            "EXPORT_DATASET",
            serde_json::json!({ "file_path": xlsx_path, "format": "xlsx" }),
        )
        .await
        .expect("Export XLSX failed");
    assert!(PathBuf::from(&xlsx_path).exists());

    let xlsx_load = daemon
        .send_request(
            "LOAD_DATASET",
            serde_json::json!({ "file_path": xlsx_path }),
        )
        .await
        .expect("Load exported XLSX failed");
    let xlsx_meta: DatasetMetadata = serde_json::from_value(xlsx_load).unwrap();
    assert_eq!(xlsx_meta.row_count, 32);
    assert_eq!(xlsx_meta.column_count, 11);

    // 4. Export ODS (OpenDocument Spreadsheet) and import back
    let ods_file = tempfile::Builder::new().suffix(".ods").tempfile().unwrap();
    let ods_path = ods_file.path().to_str().unwrap().to_string();
    daemon
        .send_request(
            "EXPORT_DATASET",
            serde_json::json!({ "file_path": ods_path, "format": "ods" }),
        )
        .await
        .expect("Export ODS failed");
    assert!(PathBuf::from(&ods_path).exists());

    let ods_load = daemon
        .send_request(
            "LOAD_DATASET",
            serde_json::json!({ "file_path": ods_path }),
        )
        .await
        .expect("Load exported ODS failed");
    let ods_meta: DatasetMetadata = serde_json::from_value(ods_load).unwrap();
    assert_eq!(ods_meta.row_count, 32);
    assert_eq!(ods_meta.column_count, 11);
}

#[tokio::test]
async fn test_project_file_save_and_open() {
    let daemon = get_test_daemon();
    daemon.ensure_started().await.expect("Failed to start daemon");

    let rrad_file = tempfile::Builder::new().suffix(".rrad").tempfile().unwrap();
    let rrad_path = rrad_file.path().to_str().unwrap().to_string();

    let sample_project = serde_json::json!({
        "$schema": "https://r-rad-studio.local/schemas/v1/project.json",
        "version": "1.0.0",
        "project_meta": {
            "title": "Cardio Study",
            "saved_at": "2026-10-25T11:45:00Z",
            "r_version": "R 4.5.3"
        },
        "dataset": {
            "dataset_name": "cardio_mock",
            "row_count": 2,
            "column_count": 2,
            "variables": [
                { "name": "age", "type": "numeric", "width": 8, "decimals": 0, "label": "Age in years", "measure": "scale", "values": [], "missing": "None", "columns": 8, "align": "right" },
                { "name": "chol", "type": "numeric", "width": 8, "decimals": 2, "label": "Serum Chol", "measure": "scale", "values": [], "missing": "None", "columns": 8, "align": "right" }
            ],
            "rows": [
                { "age": 45, "chol": 195.5 },
                { "age": 52, "chol": 220.1 }
            ]
        },
        "syntax_history": "mean(mtcars$mpg)",
        "output_logs": [
            {
                "id": "out_01",
                "timestamp": "11:00:00",
                "title": "Descriptives",
                "blocks": [
                    { "type": "syntax", "content": "mean(mtcars$mpg)" },
                    { "type": "html_table", "title": "Summary", "html": "<table><tr><td>20.09</td></tr></table>" }
                ]
            }
        ]
    });

    // Write file
    tokio::fs::write(&rrad_path, serde_json::to_string_pretty(&sample_project).unwrap())
        .await
        .expect("Failed to write project file");

    // Read back and restore to R
    let content = tokio::fs::read_to_string(&rrad_path).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed["project_meta"]["title"], "Cardio Study");

    // Restore dataset to R
    let ds = &parsed["dataset"];
    let load_res = daemon
        .send_request(
            "LOAD_RAW_DATASET",
            serde_json::json!({
                "rows": ds["rows"],
                "variables": ds["variables"],
                "dataset_name": ds["dataset_name"]
            }),
        )
        .await
        .expect("Load raw dataset into R failed");

    let meta: DatasetMetadata = serde_json::from_value(load_res).unwrap();
    assert_eq!(meta.dataset_name, "cardio_mock");
    assert_eq!(meta.row_count, 2);
    assert_eq!(meta.column_count, 2);
}

#[tokio::test]
async fn test_user_preferences_persistence() {
    use r_rad_studio_lib::ipc_commands::{get_default_preferences, get_preferences_path};

    let defaults = get_default_preferences();
    assert!(!get_preferences_path().to_string_lossy().is_empty());
    assert_eq!(defaults["theme"], "dark");
    assert_eq!(defaults["font_size_menu"], 12);
    assert_eq!(defaults["font_size_data"], 12);
    assert_eq!(defaults["font_size_output"], 13);
    assert_eq!(defaults["font_size_help"], 12);
    assert!(defaults["modules_dir"].as_str().unwrap().contains("modules"));

    // Test writing and reading back custom preferences
    let mut custom = defaults.clone();
    custom["theme"] = serde_json::json!("light");
    custom["font_size_menu"] = serde_json::json!(14);
    custom["font_family_ui"] = serde_json::json!("Roboto, sans-serif");

    let temp_pref_file = tempfile::NamedTempFile::new().unwrap();
    let pref_path = temp_pref_file.path().to_path_buf();

    tokio::fs::write(&pref_path, serde_json::to_string_pretty(&custom).unwrap())
        .await
        .unwrap();

    let content = tokio::fs::read_to_string(&pref_path).await.unwrap();
    let loaded: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(loaded["theme"], "light");
    assert_eq!(loaded["font_size_menu"], 14);
    assert_eq!(loaded["font_family_ui"], "Roboto, sans-serif");
}

#[tokio::test]
async fn test_spreadsheet_operations_and_find_replace() {
    let daemon = get_test_daemon();
    daemon.ensure_started().await.expect("Failed to start daemon");

    // 1. Load mtcars
    daemon
        .send_request("LOAD_DATASET", serde_json::json!({ "file_path": "mtcars" }))
        .await
        .expect("Load mtcars failed");

    // 2. Insert row
    let ins_res = daemon
        .send_request("INSERT_ROW", serde_json::json!({ "position": 1 }))
        .await
        .expect("Insert row failed");
    let meta_ins: DatasetMetadata = serde_json::from_value(ins_res).unwrap();
    assert_eq!(meta_ins.row_count, 33);

    // 3. Update cell at row 1, col mpg
    let upd_res = daemon
        .send_request(
            "UPDATE_CELL",
            serde_json::json!({
                "row_index": 1,
                "col_name": "mpg",
                "value": 99.9
            }),
        )
        .await
        .expect("Update cell failed");
    let meta_upd: DatasetMetadata = serde_json::from_value(upd_res).unwrap();
    assert_eq!(meta_upd.row_count, 33);

    // Verify cell value via GET_DATA_PAGE
    let page_res = daemon
        .send_request(
            "GET_DATA_PAGE",
            serde_json::json!({ "offset": 0, "limit": 1 }),
        )
        .await
        .expect("Get data page failed");
    let rows = page_res["rows"].as_array().unwrap();
    assert_eq!(rows[0]["mpg"], 99.9);

    // 4. Find and replace 99.9 -> 88.8 in column mpg
    let fr_res = daemon
        .send_request(
            "FIND_REPLACE",
            serde_json::json!({
                "find_text": "99.9",
                "replace_text": "88.8",
                "col_name": "mpg",
                "match_entire_cell": true,
                "match_case": false,
                "use_regex": false
            }),
        )
        .await
        .expect("Find and replace failed");
    assert_eq!(fr_res["replaced_count"], 1);

    // Verify replacement via GET_DATA_PAGE
    let page_res2 = daemon
        .send_request(
            "GET_DATA_PAGE",
            serde_json::json!({ "offset": 0, "limit": 1 }),
        )
        .await
        .expect("Get data page failed");
    let rows2 = page_res2["rows"].as_array().unwrap();
    assert_eq!(rows2[0]["mpg"], 88.8);

    // 5. Delete row 1
    let del_res = daemon
        .send_request("DELETE_ROW", serde_json::json!({ "row_index": 1 }))
        .await
        .expect("Delete row failed");
    let meta_del: DatasetMetadata = serde_json::from_value(del_res).unwrap();
    assert_eq!(meta_del.row_count, 32);
}

#[tokio::test]
async fn test_undo_redo_and_crash_checkpoint() {
    let daemon = get_test_daemon();
    daemon.ensure_started().await.expect("Failed to start daemon");

    // 1. Load mtcars
    daemon
        .send_request("LOAD_DATASET", serde_json::json!({ "file_path": "mtcars" }))
        .await
        .expect("Load mtcars failed");

    // 2. Initial state: verify undo status
    let status_initial = daemon
        .send_request("GET_UNDO_STATUS", serde_json::json!({}))
        .await
        .expect("Get undo status failed");
    // After load, undo stack is empty (or has 0 items because load clears stacks)
    assert_eq!(status_initial["can_undo"], false);
    assert_eq!(status_initial["can_redo"], false);

    // 3. Mutate a cell: row 1, col mpg = 999.0
    daemon
        .send_request(
            "UPDATE_CELL",
            serde_json::json!({ "row_index": 1, "col_name": "mpg", "value": 999.0 }),
        )
        .await
        .expect("Update cell failed");

    // Verify cell was changed
    let p1 = daemon
        .send_request("GET_DATA_PAGE", serde_json::json!({ "offset": 0, "limit": 1 }))
        .await
        .unwrap();
    assert_eq!(p1["rows"][0]["mpg"], 999.0);

    // Verify undo status now can undo
    let status_after_edit = daemon
        .send_request("GET_UNDO_STATUS", serde_json::json!({}))
        .await
        .unwrap();
    assert_eq!(status_after_edit["can_undo"], true);
    assert_eq!(status_after_edit["can_redo"], false);

    // 4. Perform UNDO
    let undo_res = daemon
        .send_request("UNDO", serde_json::json!({}))
        .await
        .expect("Undo failed");
    assert!(undo_res["variables"].as_array().is_some());

    // Verify original value (21.0 in mtcars) is restored
    let p_undone = daemon
        .send_request("GET_DATA_PAGE", serde_json::json!({ "offset": 0, "limit": 1 }))
        .await
        .unwrap();
    assert_eq!(p_undone["rows"][0]["mpg"], 21.0);

    // 5. Verify REDO is now available
    let status_after_undo = daemon
        .send_request("GET_UNDO_STATUS", serde_json::json!({}))
        .await
        .unwrap();
    assert_eq!(status_after_undo["can_redo"], true);

    // 6. Perform REDO
    daemon
        .send_request("REDO", serde_json::json!({}))
        .await
        .expect("Redo failed");

    // Verify value 999.0 is restored
    let p_redone = daemon
        .send_request("GET_DATA_PAGE", serde_json::json!({ "offset": 0, "limit": 1 }))
        .await
        .unwrap();
    assert_eq!(p_redone["rows"][0]["mpg"], 999.0);

    // 7. Verify atomic emergency checkpoint on disk
    let chk_status = daemon
        .send_request("CHECK_EMERGENCY_CHECKPOINT", serde_json::json!({}))
        .await
        .expect("Check emergency checkpoint failed");
    assert_eq!(chk_status["exists"], true);

    // 8. Test restore from emergency checkpoint
    let rest_res = daemon
        .send_request("RESTORE_EMERGENCY_CHECKPOINT", serde_json::json!({}))
        .await
        .expect("Restore emergency checkpoint failed");
    let rest_meta: DatasetMetadata = serde_json::from_value(rest_res).unwrap();
    assert_eq!(rest_meta.column_count, 11);
    assert_eq!(rest_meta.row_count, 32);
}

