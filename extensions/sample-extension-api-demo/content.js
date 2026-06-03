// Extension API Demo - Content Script
// This script runs on web pages and demonstrates content script capabilities

console.log('Extension API Demo content script loaded');

// Inject a demo button into the page
function injectDemoButton() {
  const button = document.createElement('button');
  button.textContent = 'Extension API Demo';
  button.style.cssText = `
    position: fixed;
    bottom: 20px;
    right: 20px;
    padding: 12px 20px;
    background: #007bff;
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    z-index: 10000;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    font-size: 14px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  `;
  
  button.addEventListener('click', () => {
    chrome.runtime.sendMessage({
      action: 'getContextMenus'
    }, (response) => {
      alert(`Extension has ${response.items.length} menu items`);
    });
  });
  
  document.body.appendChild(button);
}

// Wait for page to load
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', injectDemoButton);
} else {
  injectDemoButton();
}

// Listen for messages from background script
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  console.log('Content script received message:', request);
  
  if (request.action === 'getPageInfo') {
    sendResponse({
      url: window.location.href,
      title: document.title,
      selection: window.getSelection().toString()
    });
  }
});

// Monitor page navigation
window.addEventListener('popstate', () => {
  console.log('Page navigation detected:', window.location.href);
});

console.log('Extension API Demo content script ready');
