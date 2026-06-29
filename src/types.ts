export type OrganizeMode = 'flat' | 'month' | 'week' | 'month-week';

export interface AuthStatus {
  authenticated: boolean;
  saved_at?: string | null;
  token_server_running: boolean;
  token_server_url: string;
}

export interface TokenServerStatus {
  running: boolean;
  url: string;
  port: number;
}

export interface AppSettings {
  output_dir?: string | null;
  organize?: OrganizeMode | null;
  delay?: number | null;
  max_pages?: number | null;
  since?: string | null;
}

export interface SyncOptions {
  dir?: string | null;
  delay?: number | null;
  dry_run?: boolean | null;
  max_pages?: number | null;
  organize?: OrganizeMode | null;
  since?: string | null;
}

export interface SyncSummary {
  downloaded: number;
  skipped: number;
  filtered: number;
  failed: number;
  remote_count: number;
  pending_count: number;
}

export interface SyncProgressEvent {
  clip_id: string;
  title: string;
  status: 'downloading' | 'skipped' | 'failed' | 'done' | 'filtered';
  message?: string | null;
}

export interface SyncPreviewItem {
  id: string;
  title: string;
  display_path: string;
}

export interface SyncPreviewResult {
  items: SyncPreviewItem[];
  summary: SyncSummary;
}

export interface LibraryClip {
  id: string;
  title: string;
  created_at?: string | null;
  synced: boolean;
}

export interface LibraryListResult {
  clips: LibraryClip[];
  local_count: number;
}
