import { useCallback, useEffect, useState } from 'react';
import {
  Alert,
  Badge,
  Button,
  Card,
  Center,
  Code,
  CopyButton,
  List,
  Stack,
  Text,
  Textarea,
  TextInput,
  Title,
} from '@mantine/core';
import { authManual, authStatus, chromeExtensionPath, tokenServerStatus } from '../lib/tauri';
import { openSunoProfile } from '../lib/suno';
import { formatErrorMessage } from '../lib/labels';
import type { AuthStatus } from '../types';

interface Props {
  onAuthenticated: () => Promise<void>;
}

export default function AuthSetupPage({ onAuthenticated }: Props) {
  const [status, setStatus] = useState<AuthStatus | null>(null);
  const [extensionPath, setExtensionPath] = useState('');
  const [token, setToken] = useState('');
  const [deviceId, setDeviceId] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    const [auth, server] = await Promise.all([authStatus(), tokenServerStatus()]);
    setStatus({ ...auth, token_server_running: server.running });
    if (auth.authenticated) {
      await onAuthenticated();
    }
  }, [onAuthenticated]);

  useEffect(() => {
    void (async () => {
      try {
        const path = await chromeExtensionPath();
        setExtensionPath(path);
        await refresh();
      } catch (err) {
        setError(formatErrorMessage(err));
      }
    })();

    const timer = window.setInterval(() => {
      void refresh();
    }, 3000);

    return () => window.clearInterval(timer);
  }, [refresh]);

  async function handleManualAuth() {
    setBusy(true);
    setError(null);
    try {
      await authManual(token, deviceId || undefined);
      await refresh();
    } catch (err) {
      setError(formatErrorMessage(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <Center mih="100vh" bg="gray.0" p="lg">
      <Card withBorder padding="xl" radius="md" maw={640} w="100%">
        <Stack gap="lg">
          <div>
            <Title order={2}>Suno 認証</Title>
            <Text c="dimmed" mt="xs">
              Chrome 拡張機能または手動入力で Suno セッションを接続します。
            </Text>
          </div>

          <GroupLike>
            <Text fw={600}>接続状態</Text>
            <Badge color={status?.token_server_running ? 'green' : 'gray'}>
              トークンサーバー: {status?.token_server_running ? '稼働中' : '待機中'}
            </Badge>
            <Badge color={status?.authenticated ? 'green' : 'yellow'}>
              セッション: {status?.authenticated ? '認証済み' : '未認証'}
            </Badge>
          </GroupLike>

          <Stack gap="xs">
            <Text fw={600}>推奨: Chrome 拡張機能</Text>
            <List size="sm" spacing="xs">
              <List.Item>
                Chrome で <Code>chrome://extensions</Code> を開く
              </List.Item>
              <List.Item>「デベロッパーモード」を有効化</List.Item>
              <List.Item>「パッケージ化されていない拡張機能を読み込む」で以下を選択</List.Item>
            </List>
            {extensionPath ? (
              <CopyButton value={extensionPath}>
                {({ copied, copy }) => (
                  <Button variant="light" onClick={copy}>
                    {copied ? 'コピーしました' : '拡張機能フォルダのパスをコピー'}
                  </Button>
                )}
              </CopyButton>
            ) : null}
            <Text size="sm" c="dimmed">
              <Code>{extensionPath || 'chrome-extension/'}</Code>
            </Text>
            <List size="sm" spacing="xs" mt="sm">
              <List.Item>
                <Button variant="subtle" px={0} onClick={() => void openSunoProfile()}>
                  suno.com/me
                </Button>
                にログインすると、拡張機能が自動でトークンを送信します
              </List.Item>
            </List>
          </Stack>

          <Stack gap="xs">
            <Text fw={600}>手動入力（フォールバック）</Text>
            <Text size="sm" c="dimmed">
              DevTools の Network タブから studio-api.prod.suno.com リクエストの JWT と device-id
              をコピーしてください。
            </Text>
            <Textarea
              label="JWT (Bearer を除く)"
              minRows={3}
              value={token}
              onChange={(event) => setToken(event.currentTarget.value)}
            />
            <TextInput
              label="device-id (空欄で自動生成)"
              value={deviceId}
              onChange={(event) => setDeviceId(event.currentTarget.value)}
            />
            <Button onClick={handleManualAuth} loading={busy} disabled={busy || !token.trim()}>
              手動で認証を保存
            </Button>
          </Stack>

          {error ? <Alert color="red">{error}</Alert> : null}
        </Stack>
      </Card>
    </Center>
  );
}

function GroupLike({ children }: { children: React.ReactNode }) {
  return <Stack gap="xs">{children}</Stack>;
}
