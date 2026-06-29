import { useEffect, useState } from 'react';
import {
  Alert,
  Button,
  Card,
  Group,
  ScrollArea,
  SimpleGrid,
  Stack,
  Text,
  Title,
} from '@mantine/core';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { listenSyncProgress, settingsGet, syncCancel, syncPreview, syncRun } from '../lib/tauri';
import { formatErrorMessage } from '../lib/labels';
import type { SyncProgressEvent, SyncSummary } from '../types';

export default function SyncPage() {
  const [busy, setBusy] = useState(false);
  const [previewing, setPreviewing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [summary, setSummary] = useState<SyncSummary | null>(null);
  const [progress, setProgress] = useState<SyncProgressEvent[]>([]);
  const [pendingCount, setPendingCount] = useState(0);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;

    void listenSyncProgress((event) => {
      setProgress((current) => [event, ...current].slice(0, 200));
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      void unlisten?.();
    };
  }, []);

  async function buildOptions() {
    const settings = await settingsGet();
    return {
      dir: settings.output_dir,
      organize: settings.organize,
      delay: settings.delay,
      max_pages: settings.max_pages,
      since: settings.since,
    };
  }

  async function handlePreview() {
    setPreviewing(true);
    setError(null);
    try {
      const result = await syncPreview({ ...(await buildOptions()), dry_run: true });
      setSummary(result.summary);
      setPendingCount(result.items.length);
    } catch (err) {
      setError(formatErrorMessage(err));
    } finally {
      setPreviewing(false);
    }
  }

  async function handleSync() {
    setBusy(true);
    setError(null);
    setProgress([]);
    try {
      const result = await syncRun(await buildOptions());
      setSummary(result);
    } catch (err) {
      setError(formatErrorMessage(err));
    } finally {
      setBusy(false);
    }
  }

  async function handleCancel() {
    await syncCancel();
  }

  return (
    <Stack gap="lg">
      <div>
        <Title order={2}>同期</Title>
        <Text c="dimmed" mt="xs">
          Suno ライブラリから未ダウンロードの WAV を取得します。
        </Text>
      </div>

      <Group>
        <Button variant="light" onClick={handlePreview} loading={previewing} disabled={busy}>
          プレビュー
        </Button>
        <Button onClick={handleSync} loading={busy} disabled={previewing}>
          同期を開始
        </Button>
        <Button variant="outline" color="red" onClick={handleCancel} disabled={!busy}>
          キャンセル
        </Button>
      </Group>

      {pendingCount > 0 && !busy ? (
        <Alert color="blue">プレビュー: {pendingCount} 件がダウンロード対象です。</Alert>
      ) : null}

      {summary ? (
        <Card withBorder padding="md">
          <SimpleGrid cols={{ base: 2, sm: 3 }} spacing="sm">
            <Stat label="取得" value={summary.downloaded} />
            <Stat label="スキップ" value={summary.skipped} />
            <Stat label="除外" value={summary.filtered} />
            <Stat label="失敗" value={summary.failed} />
            <Stat label="スキャン" value={summary.remote_count} />
            <Stat label="未同期" value={summary.pending_count} />
          </SimpleGrid>
        </Card>
      ) : null}

      {progress.length > 0 ? (
        <Card withBorder padding="md">
          <Title order={4} mb="sm">
            進捗
          </Title>
          <ScrollArea h={280}>
            <Stack gap="xs">
              {progress.map((item, index) => (
                <Text key={`${item.clip_id}-${index}`} size="sm">
                  [{item.status}] {item.title}
                  {item.message ? ` - ${item.message}` : ''}
                </Text>
              ))}
            </Stack>
          </ScrollArea>
        </Card>
      ) : null}

      {error ? <Alert color="red">{error}</Alert> : null}
    </Stack>
  );
}

function Stat({ label, value }: { label: string; value: number }) {
  return (
    <div>
      <Text size="xs" c="dimmed">
        {label}
      </Text>
      <Text fw={700} size="lg">
        {value}
      </Text>
    </div>
  );
}
