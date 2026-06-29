(function () {
  'use strict';

  const MSG_TOKEN = 'SUNO_SYNC_MINI_TOKEN';
  const MSG_REFRESH = 'SUNO_SYNC_MINI_REFRESH';
  const MSG_STATUS = 'SUNO_SYNC_MINI_STATUS';

  function injectScript() {
    const script = document.createElement('script');
    script.src = chrome.runtime.getURL('injected.js');
    script.onload = function () {
      this.remove();
    };
    (document.head || document.documentElement).appendChild(script);
  }

  injectScript();

  window.addEventListener('message', function (event) {
    if (event.source !== window) return;
    const data = event.data;
    if (!data || !data.type) return;

    if (data.type === MSG_TOKEN) {
      chrome.runtime.sendMessage({
        action: 'token_received',
        token: data.token,
        deviceId: data.deviceId,
        timestamp: data.timestamp,
      });
    }

    if (data.type === MSG_STATUS) {
      chrome.runtime.sendMessage({
        action: 'status_update',
        status: data.status,
        message: data.message,
      });
    }
  });

  chrome.runtime.onMessage.addListener(function (message, _sender, sendResponse) {
    if (message.action === 'refresh_token') {
      window.postMessage({ type: MSG_REFRESH }, '*');
      sendResponse({ ok: true });
    }
    return true;
  });
})();
