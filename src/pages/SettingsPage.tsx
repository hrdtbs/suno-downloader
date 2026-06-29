import { useEffect, useState } from 'react';
import { Alert, Button, NumberInput, Select, Stack, Text, TextInput, Title } from '@mantine/core';
import { open } from '@tauri-apps/plugin-dialog';
import { settingsGet, settingsSet } from '../lib/tauri';
import { formatErrorMessage } from '../lib/labels';
import type { AppSettings, OrganizeMode } from '../types';

const organizeOptions = [
  { value: 'flat', label: 'flat (直下)' },
  { value: 'month', label: 'month (月別)' },
  { value: 'week', label: 'week (週別)' },
  { value: 'month-week', label: 'month-week (月/週)' },
];

export default function SettingsPage() {
  const [settings, setSettings] = useState<AppSettings>({});
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void (async () => {
      try {
        setSettings(await settingsGet());
      } catch (err) {
        setError(formatErrorMessage(err));
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  async function handlePickDir() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === 'string') {
      setSettings((current) => ({ ...current, output_dir: selected }));
    }
  }

  async function handleSave() {
    setSaving(true);
    setError(null);
    setMessage(null);
    try {
      await settingsSet(settings);
      setMessage('設定を保存しました。');
    } catch (err) {
      setError(formatErrorMessage(err));
    } finally {
      setSaving(false);
    }
  }

  if (loading) {
    return <Text>読み込み中...</Text>;
  }

  return (
    <Stack gap="lg" maw={560}>
      <div>
        <Title order={2}>設定</Title>
        <Text c="dimmed" mt="xs">
          同期のデフォルト動作を設定します。
        </Text>
      </div>

      <Stack gap="md">
        <TextInput
          label="出力フォルダ"
          value={settings.output_dir ?? ''}
          onChange={(event) =>
            setSettings((current) => ({ ...current, output_dir: event.currentTarget.value }))
          }
        />
        <Button variant="light" onClick={handlePickDir}>
          フォルダを選択
        </Button>

        <Select
          label="フォルダ整理 (organize)"
          data={organizeOptions}
          value={settings.organize ?? 'week'}
          onChange={(value) =>
            setSettings((current) => ({
              ...current,
              organize: (value as OrganizeMode | null) ?? 'week',
            }))
          }
        />

        <NumberInput
          label="WAV 変換リクエスト間隔 (秒)"
          min={0}
          value={settings.delay ?? 5}
          onChange={(value) =>
            setSettings((current) => ({ ...current, delay: Number(value) || 0 }))
          }
        />

        <NumberInput
          label="最大ページ数 (0 = 無制限)"
          min={0}
          value={settings.max_pages ?? 0}
          onChange={(value) =>
            setSettings((current) => ({ ...current, max_pages: Number(value) || 0 }))
          }
        />

        <TextInput
          label="since フィルタ (例: 7d, 1w)"
          placeholder="空欄で無効"
          value={settings.since ?? ''}
          onChange={(event) =>
            setSettings((current) => ({ ...current, since: event.currentTarget.value || null }))
          }
        />

        <Button onClick={handleSave} loading={saving}>
          保存
        </Button>
      </Stack>

      {message ? <Alert color="green">{message}</Alert> : null}
      {error ? <Alert color="red">{error}</Alert> : null}
    </Stack>
  );
}
