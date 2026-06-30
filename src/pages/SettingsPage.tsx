import { useCallback, useEffect, useRef, useState } from 'react';
import { Alert, Button, NumberInput, Select, Stack, Text, TextInput } from '@mantine/core';
import { open } from '@tauri-apps/plugin-dialog';
import { settingsGet, settingsSet } from '../lib/tauri';
import { formatErrorMessage } from '../lib/labels';
import { DEFAULT_SINCE_FILTER, normalizeSinceFilter, SINCE_FILTER_OPTIONS } from '../lib/defaults';
import type { AppSettings, OrganizeMode } from '../types';

const organizeOptions = [
  { value: 'flat', label: 'flat (直下)' },
  { value: 'month', label: 'month (月別)' },
  { value: 'week', label: 'week (週別)' },
  { value: 'month-week', label: 'month-week (月/週)' },
];

function withSettingsDefaults(settings: AppSettings): AppSettings {
  return {
    output_dir: settings.output_dir ?? null,
    organize: settings.organize ?? 'week',
    max_pages: settings.max_pages ?? 0,
    since: normalizeSinceFilter(settings.since),
  };
}

export default function SettingsPage() {
  const [settings, setSettings] = useState<AppSettings>({});
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const lastSavedRef = useRef<string | null>(null);
  const settingsRef = useRef(settings);

  useEffect(() => {
    settingsRef.current = settings;
  }, [settings]);

  const saveIfChanged = useCallback(async (options?: { silent?: boolean }) => {
    const current = settingsRef.current;
    const serialized = JSON.stringify(current);
    if (lastSavedRef.current === serialized) {
      return;
    }

    if (!options?.silent) {
      setSaving(true);
      setSaved(false);
      setError(null);
    }

    try {
      await settingsSet(current);
      lastSavedRef.current = serialized;
      if (!options?.silent) {
        setSaved(true);
      }
    } catch (err) {
      if (!options?.silent) {
        setError(formatErrorMessage(err));
      }
    } finally {
      if (!options?.silent) {
        setSaving(false);
      }
    }
  }, []);

  useEffect(() => {
    void (async () => {
      try {
        const loaded = withSettingsDefaults(await settingsGet());
        setSettings(loaded);
        lastSavedRef.current = JSON.stringify(loaded);
      } catch (err) {
        setError(formatErrorMessage(err));
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  useEffect(() => {
    return () => {
      void saveIfChanged({ silent: true });
    };
  }, [saveIfChanged]);

  function handleBlur() {
    void saveIfChanged();
  }

  async function handlePickDir() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === 'string') {
      const next = { ...settingsRef.current, output_dir: selected };
      setSettings(next);
      await saveIfChanged();
    }
  }

  if (loading) {
    return <Text>読み込み中...</Text>;
  }

  return (
    <Stack gap="lg" maw={560}>
      {saving ? (
        <Text size="sm" c="dimmed">
          保存中...
        </Text>
      ) : saved ? (
        <Text size="sm" c="green">
          保存済み
        </Text>
      ) : null}

      <Stack gap="md">
        <TextInput
          label="出力フォルダ"
          value={settings.output_dir ?? ''}
          onChange={(event) => {
            const output_dir = event.currentTarget.value;
            setSettings((current) => ({ ...current, output_dir }));
          }}
          onBlur={handleBlur}
        />
        <Button variant="light" onClick={handlePickDir}>
          フォルダを選択
        </Button>

        <Select
          label="フォルダ整理"
          data={organizeOptions}
          value={settings.organize ?? 'week'}
          onChange={(value) =>
            setSettings((current) => ({
              ...current,
              organize: (value as OrganizeMode | null) ?? 'week',
            }))
          }
          onBlur={handleBlur}
        />

        <NumberInput
          label="最大ページ数 (0 = 無制限)"
          min={0}
          value={settings.max_pages ?? 0}
          onChange={(value) =>
            setSettings((current) => ({ ...current, max_pages: Number(value) || 0 }))
          }
          onBlur={handleBlur}
        />

        <Select
          label="対象期間"
          data={[...SINCE_FILTER_OPTIONS]}
          value={normalizeSinceFilter(settings.since)}
          onChange={(value) =>
            setSettings((current) => ({
              ...current,
              since: value === null ? DEFAULT_SINCE_FILTER : value,
            }))
          }
          onBlur={handleBlur}
        />
      </Stack>

      {error ? <Alert color="red">{error}</Alert> : null}
    </Stack>
  );
}
