import { openUrl } from '@tauri-apps/plugin-opener';

export function openSunoProfile(): Promise<void> {
  return openUrl('https://suno.com/me');
}
