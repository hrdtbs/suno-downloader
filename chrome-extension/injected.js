/**
 * Runs in the page context on suno.com.
 * Uses Clerk for JWT and hooks fetch/XHR to capture device-id.
 */
(function () {
  'use strict';

  const MSG_TOKEN = 'SUNO_SYNC_MINI_TOKEN';
  const MSG_REFRESH = 'SUNO_SYNC_MINI_REFRESH';
  const MSG_STATUS = 'SUNO_SYNC_MINI_STATUS';
  const API_HOST = 'studio-api.prod.suno.com';

  let lastDeviceId = null;

  function postStatus(status, message) {
    window.postMessage({ type: MSG_STATUS, status, message }, '*');
  }

  function postToken(token) {
    window.postMessage(
      {
        type: MSG_TOKEN,
        token,
        deviceId: lastDeviceId,
        timestamp: Date.now(),
      },
      '*',
    );
  }

  function captureDeviceId(headers) {
    if (!headers) return;

    if (headers instanceof Headers) {
      const value = headers.get('device-id');
      if (value) lastDeviceId = value;
      return;
    }

    if (typeof headers === 'object') {
      for (const [key, value] of Object.entries(headers)) {
        if (key.toLowerCase() === 'device-id' && typeof value === 'string') {
          lastDeviceId = value;
        }
      }
    }
  }

  function wrapFetch() {
    const originalFetch = window.fetch;
    window.fetch = function (input, init) {
      const url = typeof input === 'string' ? input : input?.url;
      if (url && url.includes(API_HOST)) {
        if (init?.headers) captureDeviceId(init.headers);
      }
      return originalFetch.apply(this, arguments);
    };
  }

  function wrapXHR() {
    const originalOpen = XMLHttpRequest.prototype.open;
    const originalSetHeader = XMLHttpRequest.prototype.setRequestHeader;

    XMLHttpRequest.prototype.open = function () {
      this._sunoUrl = arguments[1];
      return originalOpen.apply(this, arguments);
    };

    XMLHttpRequest.prototype.setRequestHeader = function (name, value) {
      if (
        this._sunoUrl &&
        String(this._sunoUrl).includes(API_HOST) &&
        String(name).toLowerCase() === 'device-id'
      ) {
        lastDeviceId = value;
      }
      return originalSetHeader.apply(this, arguments);
    };
  }

  function waitForClerk(callback, maxAttempts = 30, interval = 1000) {
    let attempts = 0;

    function check() {
      attempts += 1;
      if (window.Clerk && window.Clerk.session) {
        callback(null, window.Clerk);
        return;
      }
      if (attempts >= maxAttempts) {
        callback(new Error('Clerk not found. Are you logged in to Suno?'), null);
        return;
      }
      setTimeout(check, interval);
    }

    check();
  }

  async function grabToken() {
    try {
      if (!window.Clerk || !window.Clerk.session) {
        postStatus('no_session', 'No Clerk session found. Log in to Suno first.');
        return;
      }

      const token = await window.Clerk.session.getToken();
      if (!token) {
        postStatus('no_token', 'Clerk returned no token. Try refreshing the page.');
        return;
      }

      postToken(token);
    } catch (error) {
      postStatus('error', 'Error getting token: ' + error.message);
    }
  }

  window.addEventListener('message', function (event) {
    if (event.source !== window) return;
    if (event.data && event.data.type === MSG_REFRESH) {
      grabToken();
    }
  });

  wrapFetch();
  wrapXHR();

  waitForClerk(function (err) {
    if (err) {
      postStatus('clerk_not_found', err.message);
      return;
    }
    setTimeout(grabToken, 500);
  });
})();
