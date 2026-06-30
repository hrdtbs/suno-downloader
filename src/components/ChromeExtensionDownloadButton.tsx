import { useState } from 'react';
import { Button, type ButtonProps, Text } from '@mantine/core';
import { chromeExtensionDownload } from '../lib/tauri';
import { formatErrorMessage } from '../lib/labels';

type Props = Pick<ButtonProps, 'variant' | 'size' | 'fullWidth'>;

export default function ChromeExtensionDownloadButton({
  variant = 'light',
  size,
  fullWidth,
}: Props) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [savedPath, setSavedPath] = useState<string | null>(null);

  async function handleDownload() {
    setBusy(true);
    setError(null);
    setSavedPath(null);
    try {
      const path = await chromeExtensionDownload();
      if (path) {
        setSavedPath(path);
      }
    } catch (err) {
      setError(formatErrorMessage(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <Button
        variant={variant}
        size={size}
        fullWidth={fullWidth}
        onClick={() => void handleDownload()}
        loading={busy}
        disabled={busy}
      >
        Chrome拡張をダウンロード
      </Button>
      {savedPath ? (
        <Text size="xs" c="dimmed" ta="right">
          保存しました: {savedPath}
        </Text>
      ) : null}
      {error ? (
        <Text size="xs" c="red" ta="right">
          {error}
        </Text>
      ) : null}
    </>
  );
}
