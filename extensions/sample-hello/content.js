/**
 * Exodus sample Web Extension — content script (document_start).
 * Sets a data attribute and demonstrates chrome.storage.local + chrome.tabs.
 */
(function () {
  if (document.documentElement) {
    document.documentElement.dataset.exodusSampleHello = '1';
  }

  const extId = window.chrome?.runtime?.id;
  if (!extId) return;

  if (window.chrome?.storage?.local) {
    window.chrome.storage.local.set({ lastUrl: location.href, hits: 1 }, () => {
      window.chrome.storage.local.get(['lastUrl', 'hits'], (data) => {
        console.debug('[Exodus sample-hello] storage', data);
      });
    });
  }

  if (window.chrome?.tabs?.query) {
    window.chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
      console.debug('[Exodus sample-hello] active tab', tabs);
    });
  }

  if (window.chrome?.runtime?.onMessage?.addListener) {
    window.chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
      if (message?.type === 'tab-ping') {
        sendResponse({ from: 'content', url: location.href });
        return true;
      }
      return false;
    });
  }

  if (window.chrome?.runtime?.sendMessage) {
    window.chrome.runtime.sendMessage({ type: 'ping', url: location.href }, (reply) => {
      console.debug('[Exodus sample-hello] ping reply', reply);
    });
  }
})();
