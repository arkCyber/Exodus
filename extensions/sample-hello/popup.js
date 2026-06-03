/**
 * Exodus sample Web Extension — action popup script.
 */
(function () {
  if (chrome?.storage?.local) {
    chrome.storage.local.get(['lastPing', 'lastUrl'], (data) => {
      const p = document.createElement('p');
      p.textContent = `lastPing: ${data.lastPing ?? '—'}, lastUrl: ${data.lastUrl ?? '—'}`;
      document.body.appendChild(p);
    });
  }
})();
