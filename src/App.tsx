import { Navigate, Route, Routes } from 'react-router-dom';
import { useEffect, useState } from 'react';
import { Center, Loader } from '@mantine/core';
import { authStatus, initApp } from './lib/tauri';
import { checkForAppUpdates } from './lib/updater';
import type { AuthStatus } from './types';
import AppLayout from './components/layout/AppLayout';
import AuthSetupPage from './pages/AuthSetupPage';
import SyncPage from './pages/SyncPage';
import SettingsPage from './pages/SettingsPage';

export default function App() {
  const [status, setStatus] = useState<AuthStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const authenticated = status?.authenticated ?? false;

  async function refreshStatus() {
    const next = await authStatus();
    setStatus(next);
  }

  useEffect(() => {
    void (async () => {
      await initApp();
      await refreshStatus();
      setLoading(false);
      void checkForAppUpdates();
    })();
  }, []);

  if (loading) {
    return (
      <Center h="100vh" bg="gray.0">
        <Loader />
      </Center>
    );
  }

  if (!authenticated) {
    return (
      <AuthSetupPage
        onAuthenticated={async () => {
          await refreshStatus();
        }}
      />
    );
  }

  return (
    <AppLayout status={status} onAuthUpdated={refreshStatus}>
      <Routes>
        <Route path="/" element={<Navigate to="/sync" replace />} />
        <Route path="/sync" element={<SyncPage />} />
        <Route path="/library" element={<Navigate to="/sync" replace />} />
        <Route path="/settings" element={<SettingsPage />} />
        <Route path="*" element={<Navigate to="/sync" replace />} />
      </Routes>
    </AppLayout>
  );
}
