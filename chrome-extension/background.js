const TOKEN_SERVER_URL = 'http://127.0.0.1:38946';
const REFRESH_BUFFER_SECONDS = 30;
const ALARM_NAME = 'suno_cli_token_refresh';
const POLL_ALARM = 'suno_cli_app_poll';
const STORAGE_KEY = 'suno_cli_state';
const LEGACY_STORAGE_KEY = 'suno_sync_mini_state';

let state = {
  lastToken: null,
  lastDeviceId: null,
  lastRefresh: null,
  appConnected: false,
  sunoLoggedIn: false,
  lastError: null,
};

function saveState() {
  chrome.storage.local.set({ [STORAGE_KEY]: state });
}

function loadState() {
  chrome.storage.local.get([STORAGE_KEY, LEGACY_STORAGE_KEY], (result) => {
    const saved = result[STORAGE_KEY] ?? result[LEGACY_STORAGE_KEY];
    if (saved) {
      state = { ...state, ...saved };
      if (!result[STORAGE_KEY] && result[LEGACY_STORAGE_KEY]) {
        saveState();
      }
    }
  });
}

loadState();

function parseJwt(token) {
  try {
    const base64Url = token.split('.')[1];
    const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
    const jsonPayload = decodeURIComponent(
      atob(base64)
        .split('')
        .map((c) => '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2))
        .join(''),
    );
    return JSON.parse(jsonPayload);
  } catch {
    return null;
  }
}

async function pushTokenToApp(token, deviceId) {
  try {
    const body = { token };
    if (deviceId) body.deviceId = deviceId;

    const response = await fetch(TOKEN_SERVER_URL + '/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    });

    if (response.ok) {
      state.appConnected = true;
      state.lastError = null;
      saveState();
      updateBadge('connected');
      scheduleSmartRefresh(token);
      return true;
    }

    state.appConnected = false;
    state.lastError = 'CLI returned HTTP ' + response.status;
    saveState();
    updateBadge('error');
    return false;
  } catch {
    state.appConnected = false;
    state.lastError = 'Cannot reach suno CLI. Run: suno -i';
    saveState();
    updateBadge('disconnected');
    return false;
  }
}

function scheduleSmartRefresh(token) {
  chrome.alarms.clear(ALARM_NAME);

  let delayInMinutes = 50 / 60;
  const claims = parseJwt(token);

  if (claims && claims.exp) {
    const now = Math.floor(Date.now() / 1000);
    const refreshInSeconds = claims.exp - now - REFRESH_BUFFER_SECONDS;
    if (refreshInSeconds > 5) {
      delayInMinutes = refreshInSeconds / 60;
    } else {
      delayInMinutes = 0.1;
    }
  }

  chrome.alarms.create(ALARM_NAME, { delayInMinutes });
}

async function checkAppStatus() {
  try {
    const response = await fetch(TOKEN_SERVER_URL + '/status', {
      method: 'GET',
      signal: AbortSignal.timeout(3000),
    });

    const wasConnected = state.appConnected;

    if (response.ok) {
      state.appConnected = true;
      state.lastError = null;
      if (!wasConnected && state.lastToken) {
        pushTokenToApp(state.lastToken, state.lastDeviceId);
      }
    } else {
      state.appConnected = false;
    }
  } catch {
    state.appConnected = false;
  }

  saveState();
  updateBadge(state.appConnected ? 'connected' : 'disconnected');
}

function updateBadge(status) {
  const colors = {
    connected: '#10b981',
    disconnected: '#6b7280',
    error: '#ef4444',
  };
  const texts = {
    connected: 'OK',
    disconnected: '',
    error: '!',
  };

  chrome.action.setBadgeBackgroundColor({ color: colors[status] || '#6b7280' });
  chrome.action.setBadgeText({ text: texts[status] || '' });
}

async function requestTokenRefresh() {
  const tabs = await chrome.tabs.query({
    url: ['https://suno.com/*', 'https://*.suno.com/*'],
  });

  if (tabs.length === 0) {
    state.sunoLoggedIn = false;
    state.lastError = 'No suno.com tab open';
    saveState();
    return;
  }

  for (const tab of tabs) {
    try {
      await chrome.tabs.sendMessage(tab.id, { action: 'refresh_token' });
      return;
    } catch {
      continue;
    }
  }
}

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message.action === 'token_received') {
    state.lastToken = message.token;
    state.lastDeviceId = message.deviceId || state.lastDeviceId;
    state.lastRefresh = message.timestamp || Date.now();
    state.sunoLoggedIn = true;
    saveState();

    pushTokenToApp(message.token, state.lastDeviceId).then((success) => {
      sendResponse({ success });
    });
    return true;
  }

  if (message.action === 'status_update') {
    if (message.status === 'no_session' || message.status === 'clerk_not_found') {
      state.sunoLoggedIn = false;
    }
    state.lastError = message.message;
    saveState();
  }

  if (message.action === 'get_state') {
    sendResponse(state);
    return false;
  }

  if (message.action === 'manual_refresh') {
    requestTokenRefresh();
    sendResponse({ ok: true });
    return false;
  }

  if (message.action === 'check_app') {
    checkAppStatus().then(() => sendResponse(state));
    return true;
  }
});

chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === ALARM_NAME) {
    requestTokenRefresh();
  }
  if (alarm.name === POLL_ALARM) {
    checkAppStatus();
  }
});

function startPolling() {
  chrome.alarms.create(POLL_ALARM, { periodInMinutes: 5 / 60 });
  checkAppStatus();
}

chrome.runtime.onInstalled.addListener(startPolling);
chrome.runtime.onStartup.addListener(() => {
  loadState();
  startPolling();
});
