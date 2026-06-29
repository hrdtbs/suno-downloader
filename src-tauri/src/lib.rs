mod commands;
mod config;
mod suno;
mod token_server;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use token_server::TokenServerManager;

pub struct AppState {
    pub token_server: TokenServerManager,
    pub sync_cancel: Arc<AtomicBool>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(AppState {
            token_server: TokenServerManager::new(),
            sync_cancel: Arc::new(AtomicBool::new(false)),
        })
        .invoke_handler(tauri::generate_handler![
            commands::init_app,
            commands::auth_status,
            commands::auth_manual,
            commands::auth_logout,
            commands::token_server_status,
            commands::sync_preview_cmd,
            commands::sync_run_cmd,
            commands::sync_cancel,
            commands::settings_get,
            commands::settings_set,
            commands::library_list_cmd,
            commands::chrome_extension_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
