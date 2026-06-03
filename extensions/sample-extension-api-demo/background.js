// Extension API Demo - Background Service Worker
// This file demonstrates all available Extension APIs

// Context Menus API
chrome.runtime.onInstalled.addListener(() => {
  console.log('Extension API Demo installed');
  
  // Create context menu items
  chrome.contextMenus.create({
    id: 'demo-menu-1',
    title: 'Demo Action 1',
    contexts: ['all']
  });
  
  chrome.contextMenus.create({
    id: 'demo-menu-2',
    title: 'Demo Action 2',
    contexts: ['selection']
  });
  
  chrome.contextMenus.create({
    id: 'demo-separator',
    type: 'separator',
    contexts: ['all']
  });
  
  chrome.contextMenus.create({
    id: 'demo-menu-3',
    title: 'Demo Action 3',
    contexts: ['link']
  });
});

// Context menu click handler
chrome.contextMenus.onClicked.addListener((info, tab) => {
  console.log('Context menu clicked:', info, tab);
  
  // Show notification
  chrome.notifications.create({
    type: 'basic',
    iconUrl: 'icons/icon128.png',
    title: 'Extension API Demo',
    message: `Menu item ${info.menuItemId} clicked`
  });
});

// Web Navigation API
chrome.webNavigation.onBeforeNavigate.addListener((details) => {
  console.log('Before navigate:', details.url);
});

chrome.webNavigation.onCompleted.addListener((details) => {
  console.log('Navigation completed:', details.url);
  
  // Record visit to top sites
  chrome.topSites.get((sites) => {
    console.log('Current top sites:', sites);
  });
});

// Omnibox API
chrome.omnibox.onInputChanged.addListener((text, suggest) => {
  console.log('Omnibox input changed:', text);
  
  // Provide suggestions
  suggest([
    {
      content: `search:${text}`,
      description: `Search for "${text}"`
    },
    {
      content: `navigate:${text}`,
      description: `Navigate to "${text}"`
    }
  ]);
});

chrome.omnibox.onInputEntered.addListener((text, disposition) => {
  console.log('Omnibox input entered:', text, disposition);
  
  if (text.startsWith('search:')) {
    const query = text.replace('search:', '');
    chrome.tabs.create({ url: `https://www.google.com/search?q=${encodeURIComponent(query)}` });
  } else if (text.startsWith('navigate:')) {
    const url = text.replace('navigate:', '');
    chrome.tabs.create({ url: url.startsWith('http') ? url : `https://${url}` });
  }
});

// Side Panel API
chrome.sidePanel.onPanelShown.addListener((info) => {
  console.log('Side panel shown:', info);
});

chrome.sidePanel.onPanelHidden.addListener((info) => {
  console.log('Side panel hidden:', info);
});

// Identity API (OAuth)
chrome.action.onClicked.addListener((tab) => {
  console.log('Extension icon clicked');
  
  // Example: Get authentication token
  chrome.identity.getAuthToken({ interactive: true }, (token) => {
    if (chrome.runtime.lastError) {
      console.error('Auth error:', chrome.runtime.lastError);
    } else {
      console.log('Auth token:', token);
    }
  });
});

// DevTools API
chrome.devtools.panels.create('Demo Panel', 'icons/icon16.png', 'panel.html', (panel) => {
  console.log('DevTools panel created');
  
  panel.onShown.addListener(() => {
    console.log('DevTools panel shown');
  });
  
  panel.onHidden.addListener(() => {
    console.log('DevTools panel hidden');
  });
});

// Top Sites API
chrome.topSites.get((sites) => {
  console.log('Top sites loaded:', sites);
});

// Message handling
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  console.log('Message received:', request);
  
  if (request.action === 'getContextMenus') {
    chrome.contextMenus.getAll((items) => {
      sendResponse({ items });
    });
    return true; // async response
  }
  
  if (request.action === 'getTopSites') {
    chrome.topSites.get((sites) => {
      sendResponse({ sites });
    });
    return true;
  }
  
  if (request.action === 'openSidePanel') {
    chrome.sidePanel.open({ windowId: sender.tab.windowId });
    sendResponse({ success: true });
  }
});

// Storage API for persistence
chrome.storage.local.get(['demoData'], (result) => {
  if (!result.demoData) {
    chrome.storage.local.set({
      demoData: {
        menuClicks: 0,
        navigations: 0,
        omniboxUses: 0
      }
    });
  }
});

// Track usage
chrome.contextMenus.onClicked.addListener(() => {
  chrome.storage.local.get(['demoData'], (result) => {
    const data = result.demoData || { menuClicks: 0 };
    data.menuClicks++;
    chrome.storage.local.set({ demoData: data });
  });
});

chrome.webNavigation.onCompleted.addListener(() => {
  chrome.storage.local.get(['demoData'], (result) => {
    const data = result.demoData || { navigations: 0 };
    data.navigations++;
    chrome.storage.local.set({ demoData: data });
  });
});

chrome.omnibox.onInputEntered.addListener(() => {
  chrome.storage.local.get(['demoData'], (result) => {
    const data = result.demoData || { omniboxUses: 0 };
    data.omniboxUses++;
    chrome.storage.local.set({ demoData: data });
  });
});

console.log('Extension API Demo background script loaded');
