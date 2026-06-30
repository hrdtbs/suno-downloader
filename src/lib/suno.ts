import { openUrl } from '@tauri-apps/plugin-opener';
import { authAllowExtension } from './tauri';

export async function openSunoProfile(): Promise<void> {
  await authAllowExtension();
  await openUrl('https://suno.com/me');
}
