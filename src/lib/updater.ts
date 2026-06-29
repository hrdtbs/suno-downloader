import { isTauri } from '@tauri-apps/api/core';
import { getVersion } from '@tauri-apps/api/app';
import { ask } from '@tauri-apps/plugin-dialog';
import { relaunch } from '@tauri-apps/plugin-process';
import { check } from '@tauri-apps/plugin-updater';
import { formatErrorMessage } from './labels';

export type UpdateCheckResult =
  | { kind: 'skipped' }
  | { kind: 'up_to_date' }
  | { kind: 'installed' }
  | { kind: 'declined'; version: string }
  | { kind: 'error'; message: string };

function canCheckForUpdates(): boolean {
  return isTauri() && import.meta.env.PROD;
}

function formatReleaseNotes(body: string | undefined): string {
  if (!body?.trim()) {
    return '';
  }
  return `\n\n${body.trim()}`;
}

async function installUpdate(
  update: NonNullable<Awaited<ReturnType<typeof check>>>,
): Promise<boolean> {
  const confirmed = await ask(
    `バージョン ${update.version} が利用可能です。${formatReleaseNotes(update.body ?? undefined)}今すぐ更新しますか？`,
    {
      title: 'アップデート',
      kind: 'info',
      okLabel: '更新',
      cancelLabel: '後で',
    },
  );

  if (!confirmed) {
    return false;
  }

  await update.downloadAndInstall();
  await relaunch();
  return true;
}

export async function getAppVersion(): Promise<string> {
  if (!isTauri()) {
    return import.meta.env.VITE_APP_VERSION ?? 'dev';
  }
  return getVersion();
}

export async function checkForAppUpdates(): Promise<UpdateCheckResult> {
  if (!canCheckForUpdates()) {
    return { kind: 'skipped' };
  }

  try {
    const update = await check();
    if (!update) {
      return { kind: 'up_to_date' };
    }

    const installed = await installUpdate(update);
    if (installed) {
      return { kind: 'installed' };
    }

    return { kind: 'declined', version: update.version };
  } catch (error) {
    const errorMessage = formatErrorMessage(error);
    return { kind: 'error', message: errorMessage };
  }
}
