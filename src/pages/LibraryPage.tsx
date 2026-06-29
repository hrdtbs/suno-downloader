import { useEffect, useState } from 'react';
import {
  Alert,
  Badge,
  Button,
  Card,
  Group,
  ScrollArea,
  Stack,
  Text,
  TextInput,
  Title,
} from '@mantine/core';
import { libraryList, settingsGet } from '../lib/tauri';
import { formatErrorMessage } from '../lib/labels';
import type { LibraryClip } from '../types';

export default function LibraryPage() {
  const [clips, setClips] = useState<LibraryClip[]>([]);
  const [localCount, setLocalCount] = useState(0);
  const [since, setSince] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function load() {
    setLoading(true);
    setError(null);
    try {
      const settings = await settingsGet();
      const result = await libraryList(
        settings.output_dir ?? undefined,
        since || settings.since || undefined,
        settings.max_pages ?? undefined,
      );
      setClips(result.clips);
      setLocalCount(result.local_count);
    } catch (err) {
      setError(formatErrorMessage(err));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void load();
    }, 0);
    return () => window.clearTimeout(timer);
    // eslint-disable-next-line react-hooks/exhaustive-deps -- initial load only
  }, []);

  return (
    <Stack gap="lg">
      <div>
        <Title order={2}>ライブラリ</Title>
        <Text c="dimmed" mt="xs">
          リモートの clip 一覧とローカル同期状態を表示します。
        </Text>
      </div>

      <Group align="flex-end">
        <TextInput
          label="since フィルタ (例: 7d, 1w)"
          placeholder="空欄で全件"
          value={since}
          onChange={(event) => setSince(event.currentTarget.value)}
        />
        <Button onClick={load} loading={loading}>
          再読み込み
        </Button>
      </Group>

      <Text size="sm" c="dimmed">
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

      {error ? <Alert color="red">{error}</Alert> : null}
    </Stack>
  );
}
