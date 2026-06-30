use std::sync::atomic::Ordering;

use tauri::State;
use tauri_plugin_dialog::DialogExt;
use uuid::Uuid;

use crate::chrome_extension;

use crate::config::paths::token_server_url;
use crate::config::session::{
    delete_session, load_session, normalize_token, save_session, try_load_session, AuthStatus,
};
use crate::config::settings::{load_settings, save_settings};
use crate::suno::auth::verify_session;
use crate::suno::sync::{library_list, request_sync_cancel, sync_preview, sync_run};
use crate::suno::types::{
    AppSettings, LibraryListResult, SyncOptions, SyncPreviewResult, SyncSummary, TokenServerStatus,
};
use crate::token_server::TokenServerManager;
use crate::AppState;

#[tauri::command]
pub async fn init_app(state: State<'_, AppState>) -> Result<(), String> {
    state
        .token_server
        .start()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn auth_status(_state: State<'_, AppState>) -> Result<AuthStatus, String> {
    let token_status = TokenServerManager::status().await;
    let session = try_load_session().await;

    Ok(AuthStatus {
        authenticated: session.is_some(),
        saved_at: session.map(|value| value.saved_at),
        token_server_running: token_status.running,
        token_server_url: token_server_url(),
    })
}

#[tauri::command]
pub async fn auth_manual(
    token: String,
    device_id: Option<String>,
    skip_verify: Option<bool>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let jwt = normalize_token(&token);
    if jwt.is_empty() {
        return Err("JWT is required.".to_string());
    }

    let resolved_device_id = device_id
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    if !skip_verify.unwrap_or(false) {
        verify_session(&jwt, &resolved_device_id)
            .await
            .map_err(|error| error.to_string())?;
    }

    save_session(&jwt, &resolved_device_id)
        .await
        .map_err(|error| error.to_string())?;
    state.block_extension_auth.store(false, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub async fn auth_logout(state: State<'_, AppState>) -> Result<(), String> {
    state.block_extension_auth.store(true, Ordering::Relaxed);
    delete_session().await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn auth_allow_extension(state: State<'_, AppState>) -> Result<(), String> {
    state.block_extension_auth.store(false, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub async fn token_server_status(_state: State<'_, AppState>) -> Result<TokenServerStatus, String> {
    Ok(TokenServerManager::status().await)
}

#[tauri::command]
pub async fn sync_preview_cmd(options: SyncOptions) -> Result<SyncPreviewResult, String> {
    load_session().await.map_err(|error| error.to_string())?;
    sync_preview(options).await
}

#[tauri::command]
pub async fn sync_run_cmd(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    options: SyncOptions,
) -> Result<SyncSummary, String> {
    load_session().await.map_err(|error| error.to_string())?;
    state.sync_cancel.store(false, Ordering::Relaxed);
    sync_run(app, options, state.sync_cancel.clone()).await
}

#[tauri::command]
pub async fn sync_cancel(state: State<'_, AppState>) -> Result<(), String> {
    request_sync_cancel(&state.sync_cancel);
    Ok(())
}

#[tauri::command]
pub async fn settings_get() -> Result<AppSettings, String> {
    load_settings().await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn settings_set(settings: AppSettings) -> Result<(), String> {
    save_settings(&settings)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn library_list_cmd(
    output_dir: Option<String>,
    since: Option<String>,
    max_pages: Option<u32>,
) -> Result<LibraryListResult, String> {
    load_session().await.map_err(|error| error.to_string())?;
    library_list(output_dir, since, max_pages).await
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn chrome_extension_path(app: tauri::AppHandle) -> Result<String, String> {
    chrome_extension::resolve_dir(&app).map(|path| path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn chrome_extension_download(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let ext_dir = chrome_extension::resolve_dir(&app)?;

    let dest = app
        .dialog()
        .file()
        .set_title("Chrome拡張機能を保存")
        .set_file_name("suno-downloader-chrome-extension.zip")
        .add_filter("ZIP アーカイブ", &["zip"])
        .blocking_save_file();

    let Some(dest) = dest else {
        return Ok(None);
    };

    let dest_path = dest.into_path().map_err(|error| error.to_string())?;
    chrome_extension::zip_directory(&ext_dir, &dest_path)?;
    Ok(Some(dest_path.to_string_lossy().to_string()))
}
