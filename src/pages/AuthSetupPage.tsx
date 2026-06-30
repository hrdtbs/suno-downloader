import { useCallback, useEffect, useRef, useState } from 'react';
import {
  Alert,
  Badge,
  Button,
  Card,
  Center,
  Code,
  Group,
  List,
  Stack,
  Text,
  Title,
} from '@mantine/core';
import { authStatus, tokenServerStatus } from '../lib/tauri';
import { openSunoProfile } from '../lib/suno';
import { formatErrorMessage } from '../lib/labels';
import type { AuthStatus } from '../types';
import ChromeExtensionDownloadButton from '../components/ChromeExtensionDownloadButton';

interface Props {
  onAuthenticated: () => Promise<void>;
}

export default function AuthSetupPage({ onAuthenticated }: Props) {
  const [status, setStatus] = useState<AuthStatus | null>(null);
  const [checking, setChecking] = useState(false);
  const [continuing, setContinuing] = useState(false);
  const [extensionLoginPending, setExtensionLoginPending] = useState(false);
  const [lastCheckedAt, setLastCheckedAt] = useState<Date | null>(null);
  const [error, setError] = useState<string | null>(null);
  const autoContinueStartedRef = useRef(false);

  const checkStatus = useCallback(async () => {
    const [auth, server] = await Promise.all([authStatus(), tokenServerStatus()]);
    setStatus({ ...auth, token_server_running: server.running });
    setLastCheckedAt(new Date());
    return auth.authenticated;
  }, []);

  const refreshStatus = useCallback(async () => {
    setChecking(true);
    setError(null);
    try {
      return await checkStatus();
    } catch (err) {
      setError(formatErrorMessage(err));
      return false;
    } finally {
      setChecking(false);
    }
  }, [checkStatus]);

  const handleContinue = useCallback(async () => {
    setContinuing(true);
    setError(null);
    try {
      const isAuthenticated = await checkStatus();
      if (!isAuthenticated) {
        autoContinueStartedRef.current = false;
        setError('まだ認証されていません。Chrome 拡張でログインしてください。');
        return;
      }
      await onAuthenticated();
    } catch (err) {
      autoContinueStartedRef.current = false;
      setError(formatErrorMessage(err));
    } finally {
      setContinuing(false);
    }
  }, [checkStatus, onAuthenticated]);

  useEffect(() => {
    void refreshStatus();
  }, [refreshStatus]);

  useEffect(() => {
    if (!extensionLoginPending) {
      return;
    }

    const timer = window.setInterval(() => {
      void (async () => {
        try {
          const isAuthenticated = await checkStatus();
          if (isAuthenticated) {
            setExtensionLoginPending(false);
          }
        } catch {
          // ポーリング中の一時的な失敗は無視する
        }
      })();
    }, 3000);

    return () => window.clearInterval(timer);
  }, [extensionLoginPending, checkStatus]);

  const authenticated = status?.authenticated ?? false;

  useEffect(() => {
    if (!authenticated) {
      autoContinueStartedRef.current = false;
      return;
    }
    if (autoContinueStartedRef.current) {
      return;
    }
    autoContinueStartedRef.current = true;
    void handleContinue();
  }, [authenticated, handleContinue]);

  async function handleExtensionLogin() {
    setError(null);
    try {
      await openSunoProfile();
      setExtensionLoginPending(true);
    } catch (err) {
      setError(formatErrorMessage(err));
      setExtensionLoginPending(false);
    }
  }

  return (
    <Center mih="100vh" bg="gray.0" p="lg">
      <Card withBorder padding="xl" radius="md" maw={640} w="100%">
        <Stack gap="lg">
          <div>
            <Title order={2}>Suno 認証</Title>
            <Text c="dimmed" mt="xs">
              Chrome 拡張機能で Suno セッションを接続します。
            </Text>
          </div>

          <Stack gap="sm">
            <Text fw={600}>接続状態</Text>
            <Group gap="xs">
              <Badge color={status?.token_server_running ? 'green' : 'gray'}>
                トークンサーバー: {status?.token_server_running ? '稼働中' : '待機中'}
              </Badge>
              <Badge color={authenticated ? 'green' : 'yellow'}>
                セッション: {authenticated ? '認証済み' : '未認証'}
              </Badge>
            </Group>
            {lastCheckedAt ? (
              <Text size="xs" c="dimmed">
                最終確認: {lastCheckedAt.toLocaleString()}
              </Text>
            ) : null}
            <Button
              variant="light"
              onClick={() => void refreshStatus()}
              loading={checking || continuing}
              disabled={checking || continuing}
            >
              認証状態を確認
            </Button>
            {extensionLoginPending ? (
              <Alert color="blue" title="ログイン待機中">
                suno.com
                でログインすると、拡張機能がトークンを送信します。認証が完了すると自動的にアプリへ進みます。
              </Alert>
            ) : null}
            {authenticated && continuing ? (
              <Alert color="green" title="認証済み">
                アプリへ移動しています…
              </Alert>
            ) : null}
          </Stack>

          <Stack gap="xs">
            <Text fw={600}>Chrome 拡張機能</Text>
            <ChromeExtensionDownloadButton />
            <List size="sm" spacing="xs">
              <List.Item>ZIP を展開する</List.Item>
              <List.Item>
                Chrome で <Code>chrome://extensions</Code> を開く
              </List.Item>
              <List.Item>「デベロッパーモード」を有効化</List.Item>
              <List.Item>
                「パッケージ化されていない拡張機能を読み込む」で展開したフォルダを選択
              </List.Item>
            </List>
            <Button
              onClick={() => void handleExtensionLogin()}
              disabled={extensionLoginPending || continuing}
            >
              {extensionLoginPending ? 'ログイン待機中…' : 'Chrome拡張でログイン'}
            </Button>
          </Stack>

          {error ? <Alert color="red">{error}</Alert> : null}
        </Stack>
      </Card>
    </Center>
  );
}
