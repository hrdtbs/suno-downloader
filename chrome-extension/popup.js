function render(state) {
  const cli = document.getElementById('cli');
  const suno = document.getElementById('suno');
  const errorRow = document.getElementById('error-row');
  const error = document.getElementById('error');

  cli.textContent = state.appConnected ? 'Connected' : 'Not running';
  cli.className = state.appConnected ? 'ok' : 'err';

  suno.textContent = state.sunoLoggedIn ? 'Logged in' : 'Not logged in';
  suno.className = state.sunoLoggedIn ? 'ok' : 'warn';

  if (state.lastError) {
    errorRow.hidden = false;
    error.textContent = state.lastError;
  } else {
    errorRow.hidden = true;
  }
}

chrome.runtime.sendMessage({ action: 'get_state' }, (state) => {
  render(state || {});
});

chrome.runtime.sendMessage({ action: 'check_app' }, (state) => {
  render(state || {});
});

document.getElementById('refresh').addEventListener('click', () => {
  chrome.runtime.sendMessage({ action: 'manual_refresh' }, () => {
    setTimeout(() => {
      chrome.runtime.sendMessage({ action: 'get_state' }, (state) => {
        render(state || {});
      });
    }, 1000);
  });
});
