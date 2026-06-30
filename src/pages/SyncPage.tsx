import { useCallback, useEffect, useState } from 'react';
import {
  Alert,
  Badge,
  Button,
  Card,
  Group,
  ScrollArea,
  Select,
  SimpleGrid,
  Stack,
  Text,
  Title,
} from '@mantine/core';
import type { UnlistenFn } from '@tauri-apps/api/event';
import {
  libraryList,
  listenSyncProgress,
  settingsGet,
  syncCancel,
  syncPreview,
  syncRun,
} from '../lib/tauri';
import {
  DEFAULT_SINCE_FILTER,
  normalizeSinceFilter,
  sinceFilterForApi,
  SINCE_FILTER_OPTIONS,
} from '../lib/defaults';
import { formatErrorMessage } from '../lib/labels';
import type { LibraryClip, OrganizeMode, SyncProgressEvent, SyncSummary } from '../types';

export default function SyncPage() {
  const [busy, setBusy] = useState(false);
  const [previewing, setPreviewing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [summary, setSummary] = useState<SyncSummary | null>(null);
  const [progress, setProgress] = useState<SyncProgressEvent[]>([]);
  const [pendingCount, setPendingCount] = useState(0);
  const [clips, setClips] = useState<LibraryClip[]>([]);
  const [localCount, setLocalCount] = useState(0);
  const [libraryLoading, setLibraryLoading] = useState(false);
  const [sinceFilter, setSinceFilter] = useState(DEFAULT_SINCE_FILTER);
  const [maxPages, setMaxPages] = useState<number | undefined>(undefined);
  const [outputDir, setOutputDir] = useState<string | undefined>(undefined);
  const [organize, setOrganize] = useState<OrganizeMode | undefined>(undefined);

  const loadLibrary = useCallback(async (since: string) => {
    setLibraryLoading(true);
    try {
      const settings = await settingsGet();
      const result = await libraryList(
        settings.output_dir ?? undefined,
        sinceFilterForApi(since),
        settings.max_pages ?? undefined,
      );
      setClips(result.clips);
      setLocalCount(result.local_count);
    } catch (err) {
      setError(formatErrorMessage(err));
    } finally {
      setLibraryLoading(false);
    }
  }, []);

  useEffect(() => {
    void (async () => {
      try {
        const settings = await settingsGet();
        const since = normalizeSinceFilter(settings.since);
        setSinceFilter(since);
        setMaxPages(settings.max_pages ?? undefined);
        setOutputDir(settings.output_dir ?? undefined);
        setOrganize(settings.organize ?? undefined);
        await loadLibrary(since);
      } catch (err) {
        setError(formatErrorMessage(err));
      }
    })();
  }, [loadLibrary]);

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

  function buildOptions() {
    return {
      dir: outputDir,
      organize,
      max_pages: maxPages,
      since: sinceFilter === '' ? '' : sinceFilter.trim() || null,
    };
  }

  async function handleSinceFilterChange(value: string | null) {
    const next = value === null ? DEFAULT_SINCE_FILTER : value;
    setSinceFilter(next);
    setSummary(null);
    setPendingCount(0);
    await loadLibrary(next);
  }

  async function handlePreview() {
    setPreviewing(true);
    setError(null);
    try {
      const result = await syncPreview({ ...buildOptions(), dry_run: true });
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
      const result = await syncRun(buildOptions());
      setSummary(result);
      await loadLibrary(sinceFilter);
    } catch (err) {
      setError(formatErrorMessage(err));
    } finally {
      setBusy(false);
    }
  }

  async function handleCancel() {
    await syncCancel();
  }

  const controlsDisabled = busy || previewing || libraryLoading;

  return (
    <Stack gap="lg">
      <div>
        <Title order={2}>同期</Title>
        <Text c="dimmed" mt="xs">
          Suno ライブラリから未ダウンロードの WAV を取得します。
        </Text>
      </div>

      <Group align="flex-end" wrap="wrap">
        <Select
          label="対象期間"
          data={[...SINCE_FILTER_OPTIONS]}
          value={sinceFilter}
          onChange={(value) => void handleSinceFilterChange(value)}
          disabled={controlsDisabled}
          maw={260}
        />
        <Button
          variant="light"
          onClick={handlePreview}
          loading={previewing}
          disabled={busy || libraryLoading}
        >
          プレビュー
        </Button>
        <Button onClick={handleSync} loading={busy} disabled={previewing || libraryLoading}>
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

      <div>
        <Group justify="space-between" align="flex-end" mb="sm">
          <div>
            <Title order={3}>同期状況</Title>
            <Text c="dimmed" size="sm" mt={4}>
              リモートの clip 一覧とローカル同期状態を表示します。
            </Text>
          </div>
          <Button
            variant="light"
            onClick={() => void loadLibrary(sinceFilter)}
            loading={libraryLoading}
            disabled={busy || previewing}
          >
            再読み込み
          </Button>
        </Group>

        <Text size="sm" c="dimmed" mb="sm">
          ローカル同期済み: {localCount} 件 / 表示: {clips.length} 件
        </Text>

        <Card withBorder padding="md">
          <ScrollArea h={480}>
            <Stack gap="sm">
              {clips.map((clip) => (
                <Group key={clip.id} justify="space-between" wrap="nowrap">
                  <div>
                    <Text fw={600}>{clip.title}</Text>
                    <Text size="xs" c="dimmed">
                      {clip.created_at ? new Date(clip.created_at).toLocaleString() : '日付不明'} ·{' '}
                      {clip.id}
                    </Text>
                  </div>
                  <Badge color={clip.synced ? 'green' : 'orange'}>
                    {clip.synced ? '同期済み' : '未同期'}
                  </Badge>
                </Group>
              ))}
            </Stack>
          </ScrollArea>
        </Card>
      </div>

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
