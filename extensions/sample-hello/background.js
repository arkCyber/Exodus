/**
 * Exodus sample Web Extension — background service worker (MV3).
 */
(function () {
  const extId = chrome?.runtime?.id;
  if (!extId) return;

  chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
    console.debug('[Exodus sample-hello/bg]', message);
    if (message?.type === 'ping') {
      if (chrome.storage?.local) {
        chrome.storage.local.set({ lastPing: Date.now() });
      }
      sendResponse({ pong: true, at: Date.now() });
      return true;
    }
    sendResponse({ ok: false });
    return false;
  });

  if (chrome.storage?.local) {
    chrome.storage.local.get(['lastPing'], (data) => {
      console.debug('[Exodus sample-hello/bg] storage', data);
    });
  }

  if (chrome.tabs?.query && chrome.tabs?.sendMessage) {
    chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
      const tab = tabs && tabs[0];
      if (!tab || tab.id == null) return;
      chrome.tabs.sendMessage(tab.id, { type: 'tab-ping' }, (reply) => {
        console.debug('[Exodus sample-hello/bg] tab-ping reply', reply);
      });
    });
  }
})();
