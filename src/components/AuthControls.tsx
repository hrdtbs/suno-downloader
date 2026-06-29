import { useState } from 'react';
import { Button, Stack, Text } from '@mantine/core';
import { authLogout } from '../lib/tauri';
import { formatErrorMessage } from '../lib/labels';
import type { AuthStatus } from '../types';

interface Props {
  status: AuthStatus | null;
  onUpdated: () => Promise<void>;
}

export default function AuthControls({ status, onUpdated }: Props) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleLogout() {
    setBusy(true);
    setError(null);
    try {
      await authLogout();
      await onUpdated();
    } catch (err) {
      setError(formatErrorMessage(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <Stack gap="xs" align="flex-end" maw={320}>
      {status?.authenticated ? (
        <>
          <Text size="sm" ta="right">
            認証済み
            {status.saved_at ? ` (${new Date(status.saved_at).toLocaleString()})` : ''}
          </Text>
          <Button variant="outline" onClick={handleLogout} loading={busy} disabled={busy}>
            ログアウト
          </Button>
        </>
      ) : (
        <Text size="sm" c="dimmed" ta="right">
          未認証
        </Text>
      )}
      {error ? (
        <Text size="xs" c="red" ta="right">
          {error}
        </Text>
      ) : null}
    </Stack>
  );
}
