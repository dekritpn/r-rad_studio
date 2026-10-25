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
