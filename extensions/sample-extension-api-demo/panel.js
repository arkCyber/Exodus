// Extension API Demo - Panel Script

console.log('Extension API Demo panel loaded');

// Listen for messages from background script
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  console.log('Panel received message:', request);
  
  if (request.action === 'updatePanel') {
    // Update panel content
    document.body.innerHTML = `
      <h1>Extension API Demo Panel</h1>
      <div class="info">
        <p>${request.message}</p>
      </div>
    `;
  }
});

// Example: Monitor navigation events
chrome.webNavigation.onCompleted.addListener((details) => {
  console.log('Panel detected navigation:', details.url);
});
