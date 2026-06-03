# Extension API Demo

A comprehensive demonstration extension showcasing all available Extension APIs in Exodus Browser.

## Features

This extension demonstrates the following Extension APIs:

### 1. Context Menus API
- Creates custom context menu items
- Handles menu item clicks
- Shows notifications on interaction

### 2. Web Navigation API
- Monitors navigation events
- Tracks page loads
- Records navigation statistics

### 3. Side Panel API
- Opens and manages side panels
- Displays extension-specific content
- Integrates with DevTools

### 4. Omnibox API
- Adds address bar suggestions
- Handles keyword commands (prefix: "demo")
- Provides search and navigation shortcuts

### 5. Identity API
- Demonstrates OAuth authentication
- Manages authentication tokens
- Shows user profile information

### 6. Top Sites API
- Accesses most visited websites
- Displays site statistics
- Tracks visit counts

### 7. DevTools API
- Creates custom DevTools panels
- Integrates with developer tools
- Provides debugging utilities

## Installation

1. Copy this extension folder to your Exodus Browser extensions directory
2. Enable the extension in the browser settings
3. Grant necessary permissions when prompted

## Usage

### Popup Interface
Click the extension icon to access:
- Context menu management
- Side panel controls
- Top sites viewer
- Usage statistics

### Omnibox Commands
Type "demo" followed by your query in the address bar:
- `demo search <query>` - Search the web
- `demo navigate <url>` - Navigate to a URL

### Context Menu
Right-click anywhere on a page to see custom menu items:
- Demo Action 1 (all contexts)
- Demo Action 2 (selection only)
- Demo Action 3 (links only)

### Side Panel
Click "Open Panel" in the popup to open the side panel, which displays:
- Extension information
- Real-time status
- Navigation monitoring

### DevTools Panel
Open DevTools to see the custom "Demo Panel" for:
- Extension debugging
- Performance monitoring
- API testing

## Usage Statistics

The extension tracks:
- Menu item clicks
- Page navigations
- Omnibox uses

View these statistics in the popup interface.

## Files

- `manifest.json` - Extension configuration
- `background.js` - Service worker with API implementations
- `popup.html` - Popup interface
- `popup.js` - Popup logic
- `panel.html` - Side panel interface
- `panel.js` - Panel logic
- `content.js` - Content script for page injection
- `icons/` - Extension icons (to be added)

## Development

To modify this extension:

1. Edit the relevant files
2. Reload the extension in Exodus Browser
3. Test the changes

## API Examples

### Context Menus
```javascript
chrome.contextMenus.create({
  id: 'my-menu',
  title: 'My Action',
  contexts: ['all']
});
```

### Web Navigation
```javascript
chrome.webNavigation.onCompleted.addListener((details) => {
  console.log('Navigation completed:', details.url);
});
```

### Side Panel
```javascript
chrome.sidePanel.open({ windowId: tab.windowId });
```

### Omnibox
```javascript
chrome.omnibox.onInputChanged.addListener((text, suggest) => {
  suggest([{ content: text, description: 'Suggestion' }]);
});
```

### Top Sites
```javascript
chrome.topSites.get((sites) => {
  console.log('Top sites:', sites);
});
```

## License

This is a demonstration extension for Exodus Browser.
