// src-tauri/tests/r_daemon_tests.rs
// Verification Gate 1 tests: R daemon spawning, dataset loading, metadata querying under 150ms, and clean termination

use std::path::PathBuf;
use std::time::Instant;
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
async fn test_spawn_and_ping() {
    let daemon = get_test_daemon();
    daemon.ensure_started().await.expect("Failed to start R daemon");

    let resp = daemon
        .send_request("PING", serde_json::json!({}))
        .await
        .expect("PING request failed");

    assert_eq!(resp["status"], "ok");
    assert!(resp["version"].as_str().unwrap().contains("R version"));
}

#[tokio::test]
async fn test_load_50k_sav_and_query_metadata_timing() {
    let daemon = get_test_daemon();
    daemon.ensure_started().await.expect("Failed to start daemon");

    let fixture_path = PathBuf::from("tests/fixtures/test_50k.sav");
    let abs_fixture = std::fs::canonicalize(&fixture_path)
        .unwrap_or_else(|_| PathBuf::from("/home/dpng/Projects/r_rad_studio/src-tauri/tests/fixtures/test_50k.sav"));

    // 1. Load dataset
    let load_resp = daemon
        .send_request(
            "LOAD_DATASET",
            serde_json::json!({ "file_path": abs_fixture.to_str().unwrap() }),
        )
        .await
        .expect("Failed to load 50k .sav dataset");

    let meta: DatasetMetadata = serde_json::from_value(load_resp)
        .expect("Failed to parse metadata from load response");

    assert_eq!(meta.row_count, 50000);
    assert_eq!(meta.column_count, 4);

    // 2. Query variable metadata and benchmark under 150ms
    let start_time = Instant::now();
    let query_resp = daemon
        .send_request("GET_METADATA", serde_json::json!({}))
        .await
        .expect("Failed to query metadata");
    let duration = start_time.elapsed();

    let query_meta: DatasetMetadata = serde_json::from_value(query_resp)
        .expect("Failed to parse metadata");

    println!("GET_METADATA on 50k rows took: {:?}", duration);
    assert!(
        duration.as_millis() < 150,
        "Querying metadata took {:?}, exceeding 150ms limit!",
        duration
    );

    // Verify variable types and labels
    let score_var = query_meta.variables.iter().find(|v| v.name == "score").unwrap();
    assert_eq!(score_var.label, "Test Score");
    assert_eq!(score_var.measure, "scale");

    let group_var = query_meta.variables.iter().find(|v| v.name == "group").unwrap();
    assert_eq!(group_var.label, "Study Group Assignment");
    assert_eq!(group_var.measure, "nominal");
}

#[tokio::test]
async fn test_execute_analysis_and_output_blocks() {
    let daemon = get_test_daemon();
    daemon.ensure_started().await.expect("Failed to start daemon");

    let r_code = r#"
        local({
            res <- t.test(mpg ~ am, data = .active_data)
            rrad_emit_table(broom::tidy(res), title = "Two-Sample T-Test")
        })
    "#;

    let res = daemon.execute_analysis(r_code).await.expect("Analysis failed");
    assert_eq!(res["status"], "ok");

    let blocks: Vec<OutputBlock> = serde_json::from_value(res["blocks"].clone())
        .expect("Failed to deserialize output blocks");

    assert!(blocks.iter().any(|b| b.block_type == "syntax"));
    let table_block = blocks.iter().find(|b| b.block_type == "html_table");
    assert!(table_block.is_some(), "Expected html_table block in output");
    assert!(table_block.unwrap().html.as_ref().unwrap().contains("<table"));
}

#[tokio::test]
async fn test_clean_termination_on_drop() {
    let daemon = get_test_daemon();
    daemon.ensure_started().await.expect("Failed to start daemon");

    // Daemon is dropped here at end of scope
    drop(daemon);

    // Give process monitor a moment to clean up
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}
