import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  AppSettings,
  AuthStatus,
  LibraryListResult,
  SyncOptions,
  SyncPreviewResult,
  SyncProgressEvent,
  SyncSummary,
  TokenServerStatus,
} from '../types';

export async function initApp(): Promise<void> {
  await invoke('init_app');
}

export async function authStatus(): Promise<AuthStatus> {
  return invoke<AuthStatus>('auth_status');
}

export async function authManual(
  token: string,
  deviceId?: string,
  skipVerify?: boolean,
): Promise<void> {
  await invoke('auth_manual', { token, deviceId, skipVerify });
}

export async function authLogout(): Promise<void> {
  await invoke('auth_logout');
}

export async function tokenServerStatus(): Promise<TokenServerStatus> {
  return invoke<TokenServerStatus>('token_server_status');
}

export async function syncPreview(options: SyncOptions): Promise<SyncPreviewResult> {
  return invoke<SyncPreviewResult>('sync_preview_cmd', { options });
}

export async function syncRun(options: SyncOptions): Promise<SyncSummary> {
  return invoke<SyncSummary>('sync_run_cmd', { options });
}

export async function syncCancel(): Promise<void> {
  await invoke('sync_cancel');
}

export async function settingsGet(): Promise<AppSettings> {
  return invoke<AppSettings>('settings_get');
}

export async function settingsSet(settings: AppSettings): Promise<void> {
  await invoke('settings_set', { settings });
}

export async function libraryList(
  outputDir?: string,
  since?: string,
  maxPages?: number,
): Promise<LibraryListResult> {
  return invoke<LibraryListResult>('library_list_cmd', {
    outputDir,
    since,
    maxPages,
  });
}

export async function chromeExtensionPath(): Promise<string> {
  return invoke<string>('chrome_extension_path');
}

export function listenSyncProgress(
  handler: (event: SyncProgressEvent) => void,
): Promise<UnlistenFn> {
  return listen<SyncProgressEvent>('sync-progress', (event) => {
    handler(event.payload);
  });
}
