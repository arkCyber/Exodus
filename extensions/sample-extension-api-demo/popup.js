// Extension API Demo - Popup Script

document.addEventListener('DOMContentLoaded', () => {
  // Load usage statistics
  loadStats();
  
  // Context Menus
  document.getElementById('createMenu').addEventListener('click', () => {
    chrome.contextMenus.create({
      id: `demo-menu-${Date.now()}`,
      title: 'Demo Menu Item',
      contexts: ['all']
    }, () => {
      if (chrome.runtime.lastError) {
        console.error('Error creating menu:', chrome.runtime.lastError);
      } else {
        console.log('Menu item created');
      }
    });
  });
  
  document.getElementById('getMenus').addEventListener('click', () => {
    chrome.runtime.sendMessage({ action: 'getContextMenus' }, (response) => {
      console.log('Menu items:', response.items);
    });
  });
  
  // Side Panel
  document.getElementById('openPanel').addEventListener('click', () => {
    chrome.runtime.sendMessage({ action: 'openSidePanel' }, (response) => {
      console.log('Panel opened:', response);
    });
  });
  
  document.getElementById('closePanel').addEventListener('click', () => {
    chrome.sidePanel.close();
  });
  
  // Top Sites
  document.getElementById('getTopSites').addEventListener('click', () => {
    chrome.runtime.sendMessage({ action: 'getTopSites' }, (response) => {
      console.log('Top sites:', response.sites);
    });
  });
});

function loadStats() {
  chrome.storage.local.get(['demoData'], (result) => {
    const data = result.demoData || { menuClicks: 0, navigations: 0, omniboxUses: 0 };
    
    document.getElementById('menuClicks').textContent = data.menuClicks;
    document.getElementById('navigations').textContent = data.navigations;
    document.getElementById('omniboxUses').textContent = data.omniboxUses;
  });
}
