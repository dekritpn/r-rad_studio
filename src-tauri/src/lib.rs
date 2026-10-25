// src-tauri/src/lib.rs
// Tauri v2 application initialization and command registration

pub mod data_bridge;
pub mod form_registry;
pub mod ipc_commands;
pub mod r_daemon;

use std::sync::Arc;
use form_registry::FormRegistry;
use ipc_commands::AppState;
use r_daemon::{RDaemon, RDaemonConfig};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let r_daemon = Arc::new(RDaemon::new(RDaemonConfig::default()));
    let form_registry = Arc::new(FormRegistry::new());

    let state = AppState {
        r_daemon,
        form_registry: Arc::clone(&form_registry),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .setup(move |_app| {
            let registry_clone = Arc::clone(&form_registry);
            tauri::async_runtime::spawn(async move {
                let _ = registry_clone.reload().await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc_commands::ping_r,
            ipc_commands::load_dataset,
            ipc_commands::get_dataset_metadata,
            ipc_commands::update_variable_metadata,
            ipc_commands::get_data_page,
            ipc_commands::execute_analysis,
            ipc_commands::cancel_analysis,
            ipc_commands::get_forms_menu,
            ipc_commands::get_form_by_id,
            ipc_commands::save_custom_form,
            ipc_commands::rename_variable,
            ipc_commands::add_variable,
            ipc_commands::compute_variable,
            ipc_commands::export_dataset,
            ipc_commands::save_project,
            ipc_commands::open_project,
            ipc_commands::pick_file,
            ipc_commands::save_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running R-RAD Studio application");
}
