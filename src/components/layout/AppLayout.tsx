import { Box, Container, Group, Title } from '@mantine/core';
import { Link, useLocation } from 'react-router-dom';
import AuthControls from '../AuthControls';
import type { AuthStatus } from '../../types';

const navItems = [
  { to: '/sync', label: '同期' },
  { to: '/library', label: 'ライブラリ' },
  { to: '/settings', label: '設定' },
];

interface Props {
  status: AuthStatus | null;
  onAuthUpdated: () => Promise<void>;
  children: React.ReactNode;
}

export default function AppLayout({ status, onAuthUpdated, children }: Props) {
  const location = useLocation();

  return (
    <Box mih="100vh" bg="gray.0">
      <Box
        component="header"
        pos="sticky"
        top={0}
        bg="white"
        style={{
          zIndex: 100,
          borderBottom: '1px solid var(--mantine-color-gray-3)',
        }}
      >
        <Container size="lg" py="md">
          <Group justify="space-between" align="flex-start" wrap="nowrap">
            <div>
              <Title order={2}>Suno Downloader</Title>
              <Group component="nav" gap="xs" mt="sm" aria-label="メインナビゲーション">
                {navItems.map((item) => {
                  const active = location.pathname === item.to;
                  return (
                    <Link
                      key={item.to}
                      to={item.to}
                      style={{
                        textDecoration: 'none',
                        color: active ? 'white' : 'var(--mantine-color-gray-7)',
                        background: active ? 'var(--mantine-color-dark-7)' : 'transparent',
                        padding: '0.35rem 0.75rem',
                        borderRadius: 'var(--mantine-radius-xl)',
                        fontSize: 'var(--mantine-font-size-sm)',
                        fontWeight: active ? 600 : 400,
                      }}
                    >
                      {item.label}
                    </Link>
                  );
                })}
              </Group>
            </div>
            <AuthControls status={status} onUpdated={onAuthUpdated} />
          </Group>
        </Container>
      </Box>

      <Box component="main" p="lg">
        <Container size="lg">{children}</Container>
      </Box>
    </Box>
  );
}
