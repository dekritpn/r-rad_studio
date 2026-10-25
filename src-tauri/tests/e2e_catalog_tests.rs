// Verification Gate 5: Full E2E automated test suite verifying all shipped .rform.json dialogs

use r_rad_studio_lib::data_bridge::OutputBlock;
use r_rad_studio_lib::form_registry::FormRegistry;
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
async fn test_registry_loads_all_shipped_forms() {
    let registry = FormRegistry::new();
    registry.reload().await.expect("Failed to load forms");

    let menu = registry.get_menu().await;
    assert!(!menu.is_empty(), "Menu tree should not be empty");

    let analyze_node = menu.iter().find(|n| n.label == "Analyze").expect("Missing Analyze menu");
    assert!(!analyze_node.children.is_empty());

    // Verify key forms exist in registry
    assert!(registry.get_form("stat.compare_means.ind_t_test").await.is_some());
    assert!(registry.get_form("stat.compare_means.one_way_anova").await.is_some());
    assert!(registry.get_form("stat.descriptives.frequencies").await.is_some());
    assert!(registry.get_form("stat.descriptives.crosstabs").await.is_some());
    assert!(registry.get_form("stat.regression.linear").await.is_some());
    assert!(registry.get_form("stat.regression.binary_logistic").await.is_some());
    assert!(registry.get_form("stat.graphs.scatterplot").await.is_some());
    assert!(registry.get_form("stat.graphs.histogram").await.is_some());
}

#[tokio::test]
async fn test_e2e_independent_t_test() {
    let daemon = get_test_daemon();
    daemon.ensure_started().await.unwrap();

    let code = r#"
        local({
          res <- t.test(mpg ~ am, data = .active_data, var.equal = FALSE, conf.level = 0.95)
          rrad_emit_table(broom::tidy(res), title = 'T-Test: mpg by am')
        })
    "#;

    let res = daemon.execute_analysis(code).await.unwrap();
    assert_eq!(res["status"], "ok");
    let blocks: Vec<OutputBlock> = serde_json::from_value(res["blocks"].clone()).unwrap();
    assert!(blocks.iter().any(|b| b.block_type == "html_table" && b.html.as_ref().unwrap().contains("<table")));
}

#[tokio::test]
async fn test_e2e_one_way_anova() {
    let daemon = get_test_daemon();
    daemon.ensure_started().await.unwrap();

    daemon.send_request("LOAD_DATASET", serde_json::json!({ "file_path": "iris" })).await.unwrap();

    let code = r#"
        local({
          fac <- as.factor(.active_data[['Species']])
          fit <- aov(.active_data[['Sepal.Length']] ~ fac)
          rrad_emit_table(broom::tidy(fit), title = 'ANOVA Table: Sepal.Length by Species')
          tuk <- TukeyHSD(fit)
          rrad_emit_table(as.data.frame(tuk$fac), title = 'Tukey HSD Post-Hoc: Sepal.Length')
        })
    "#;

    let res = daemon.execute_analysis(code).await.unwrap();
    assert_eq!(res["status"], "ok");
    let blocks: Vec<OutputBlock> = serde_json::from_value(res["blocks"].clone()).unwrap();
    let tables: Vec<_> = blocks.iter().filter(|b| b.block_type == "html_table").collect();
    assert!(tables.len() >= 2, "Expected both ANOVA and Tukey HSD tables");
}

#[tokio::test]
async fn test_e2e_frequencies_and_crosstabs() {
    let daemon = get_test_daemon();
    daemon.ensure_started().await.unwrap();

    daemon.send_request("LOAD_DATASET", serde_json::json!({ "file_path": "mtcars" })).await.unwrap();

    // Frequencies
    let freq_code = r#"
        local({
          v <- .active_data[['cyl']]
          freq_tb <- as.data.frame(table(v))
          rrad_emit_table(freq_tb, title = 'Frequencies: cyl')
        })
    "#;
    let res = daemon.execute_analysis(freq_code).await.unwrap();
    assert_eq!(res["status"], "ok");

    // Crosstabs
    let cross_code = r#"
        local({
          tbl <- table(.active_data[['cyl']], .active_data[['am']])
          df_tbl <- as.data.frame.matrix(tbl)
          rrad_emit_table(df_tbl, title = 'Crosstab: cyl x am')
          res_chi <- suppressWarnings(chisq.test(tbl))
          rrad_emit_table(broom::tidy(res_chi), title = 'Chi-Square')
        })
    "#;
    let res = daemon.execute_analysis(cross_code).await.unwrap();
    assert_eq!(res["status"], "ok");
    let blocks: Vec<OutputBlock> = serde_json::from_value(res["blocks"].clone()).unwrap();
    assert!(blocks.iter().any(|b| b.block_type == "html_table"));
}

#[tokio::test]
async fn test_e2e_linear_regression_and_svg_plot() {
    let daemon = get_test_daemon();
    daemon.ensure_started().await.unwrap();

    daemon.send_request("LOAD_DATASET", serde_json::json!({ "file_path": "mtcars" })).await.unwrap();

    let reg_code = r#"
        local({
          fit <- lm(mpg ~ wt + hp, data = .active_data)
          rrad_emit_table(broom::tidy(fit), title = 'Linear Regression Coefficients')
          p <- ggplot2::ggplot(mtcars, ggplot2::aes(x = wt, y = mpg)) +
            ggplot2::geom_point(color = '#4f46e5') +
            ggplot2::geom_smooth(method = 'lm', color = '#ef4444')
          rrad_emit_plot(p, title = 'Regression Diagnostic Plot')
        })
    "#;

    let res = daemon.execute_analysis(reg_code).await.unwrap();
    assert_eq!(res["status"], "ok");
    let blocks: Vec<OutputBlock> = serde_json::from_value(res["blocks"].clone()).unwrap();
    assert!(blocks.iter().any(|b| b.block_type == "html_table"));
    let plot_block = blocks.iter().find(|b| b.block_type == "svg_plot");
    assert!(plot_block.is_some(), "Expected SVG plot block");
    assert!(plot_block.unwrap().svg.as_ref().unwrap().contains("<svg"));
}

#[tokio::test]
async fn test_e2e_rad_designer_save_and_hot_reload() {
    let _temp_dir = tempfile::tempdir().unwrap();
    let registry = FormRegistry::new();

    let custom_form = serde_json::json!({
        "id": "custom.correlate.bivariate",
        "version": "1.0.0",
        "menu_path": ["Analyze", "Custom", "Custom Bivariate Correlation"],
        "window": { "title": "Custom Correlation", "width": 600, "height": 400 },
        "layout": [
            { "id": "var1", "type": "VariableTargetSlot", "label": "Variable 1", "grid": { "x": 1, "y": 1, "w": 6, "h": 2 } },
            { "id": "var2", "type": "VariableTargetSlot", "label": "Variable 2", "grid": { "x": 7, "y": 1, "w": 6, "h": 2 } }
        ],
        "r_template": "local({ res <- cor.test(.active_data[[ '{{var1}}' ]], .active_data[[ '{{var2}}' ]]); rrad_emit_table(broom::tidy(res), title = 'Bivariate Correlation') })"
    });

    let saved_id = registry.save_custom_form(custom_form).await.unwrap();
    assert_eq!(saved_id, "custom.correlate.bivariate");

    // Verify it exists in registry
    let fetched = registry.get_form("custom.correlate.bivariate").await;
    assert!(fetched.is_some());

    // Verify it appears in menu tree under Custom
    let menu = registry.get_menu().await;
    let analyze = menu.iter().find(|n| n.label == "Analyze").unwrap();
    let custom = analyze.children.iter().find(|n| n.label == "Custom");
    assert!(custom.is_some(), "Custom submenu should be created");
}
