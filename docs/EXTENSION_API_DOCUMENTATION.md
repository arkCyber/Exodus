# Exodus Browser Extension API Documentation

## Overview

This document provides comprehensive documentation for the newly implemented Extension APIs in Exodus Browser. These APIs follow Chrome Extension Manifest V3 standards and provide extensions with powerful capabilities while maintaining privacy and security.

## Table of Contents

1. [Context Menus API](#context-menus-api)
2. [Web Navigation API](#web-navigation-api)
3. [Side Panel API](#side-panel-api)
4. [Omnibox API](#omnibox-api)
5. [Identity API](#identity-api)
6. [Top Sites API](#top-sites-api)
7. [DevTools API](#devtools-api)
8. [Site Isolation API](#site-isolation-api)
9. [Error Handling](#error-handling)
10. [Performance Metrics](#performance-metrics)

---

## Context Menus API

### Overview

The Context Menus API allows extensions to add items to the browser's context menu (right-click menu). Extensions can customize when and where their menu items appear.

### Methods

#### `context_menus_create`

Creates a new context menu item.

**Parameters:**
- `extensionId` (string): The ID of the extension
- `createProperties` (object):
  - `title` (string): The text to display
  - `type` (string): 'normal' | 'checkbox' | 'radio' | 'separator'
  - `enabled` (boolean, optional): Whether the item is enabled
  - `checked` (boolean, optional): Whether the item is checked
  - `parentId` (string, optional): Parent item ID for nested menus
  - `contexts` (array): Contexts where item appears
  - `iconUrl` (string, optional): Icon URL
  - `onclick` (string, optional): Click handler

**Returns:** `ContextMenuItem`

**Example:**
```typescript
const item = await window.__EXODUS_TAURI_INVOKE__('context_menus_create', {
  extensionId: 'my-extension',
  createProperties: {
    title: 'My Action',
    type: 'normal',
    contexts: ['all']
  }
});
```

#### `context_menus_update`

Updates an existing context menu item.

**Parameters:**
- `itemId` (string): The ID of the item to update
- `updateProperties` (object):
  - `title` (string, optional): New title
  - `enabled` (boolean, optional): New enabled state
  - `checked` (boolean, optional): New checked state
  - `iconUrl` (string, optional): New icon URL

**Returns:** `ContextMenuItem`

#### `context_menus_remove`

Removes a context menu item.

**Parameters:**
- `itemId` (string): The ID of the item to remove

#### `context_menus_get_all`

Gets all menu items for an extension.

**Parameters:**
- `extensionId` (string): The extension ID

**Returns:** `ContextMenuItem[]`

#### `context_menus_get_for_context`

Gets menu items for a specific context.

**Parameters:**
- `context` (string): Context type ('all', 'page', 'selection', 'link', etc.)

**Returns:** `ContextMenuItem[]`

---

## Web Navigation API

### Overview

The Web Navigation API allows extensions to monitor and control navigation events in the browser.

### Methods

#### `web_navigation_add_listener`

Adds a listener for navigation events.

**Parameters:**
- `extensionId` (string): The extension ID
- `eventTypes` (array): Event types to listen for
  - 'beforeNavigate'
  - 'committed'
  - 'completed'
  - 'errorOccurred'
  - 'createdNavigationTarget'
  - 'dominantContentChanged'
  - 'historyStateUpdated'
  - 'referenceFragmentUpdated'

#### `web_navigation_remove_listener`

Removes a navigation event listener.

**Parameters:**
- `extensionId` (string): The extension ID

#### `web_navigation_get_frame`

Gets information about a specific frame.

**Parameters:**
- `tabId` (number): The tab ID
- `frameId` (number): The frame ID

**Returns:** `NavigationDetails`

#### `web_navigation_get_all_frames`

Gets all frames for a tab.

**Parameters:**
- `tabId` (number): The tab ID

**Returns:** `NavigationDetails[]`

#### `web_navigation_get_history`

Gets navigation history with optional filtering.

**Parameters:**
- `filter` (object, optional):
  - `tabIds` (array, optional): Filter by tab IDs
  - `urlPatterns` (array, optional): Filter by URL patterns
  - `frameTypes` (array, optional): Filter by frame types

**Returns:** `NavigationEvent[]`

---

## Side Panel API

### Overview

The Side Panel API allows extensions to create and manage side panels that appear alongside the main content.

### Methods

#### `side_panel_set_options`

Sets side panel options for an extension.

**Parameters:**
- `extensionId` (string): The extension ID
- `options` (object):
  - `path` (string, optional): Path to panel page
  - `title` (string, optional): Panel title
  - `behavior` (string, optional): 'openPanel' | 'dontOpenPanel'

**Returns:** `SidePanelConfiguration`

#### `side_panel_open`

Opens the side panel for an extension.

**Parameters:**
- `extensionId` (string): The extension ID

**Returns:** `SidePanelState`

#### `side_panel_close`

Closes the side panel for an extension.

**Parameters:**
- `extensionId` (string): The extension ID

**Returns:** `SidePanelState`

#### `side_panel_get_state`

Gets the current state of a side panel.

**Parameters:**
- `extensionId` (string): The extension ID

**Returns:** `SidePanelState`

#### `side_panel_enable`

Enables a side panel for an extension.

**Parameters:**
- `extensionId` (string): The extension ID

#### `side_panel_disable`

Disables a side panel for an extension.

**Parameters:**
- `extensionId` (string): The extension ID

---

## Omnibox API

### Overview

The Omnibox API allows extensions to add suggestions to the browser's address bar.

### Methods

#### `omnibox_set_default_suggestion`

Sets the default suggestion for an extension.

**Parameters:**
- `extensionId` (string): The extension ID
- `suggestion` (object):
  - `description` (string): Suggestion description

#### `omnibox_set_keyword`

Sets the keyword for an extension's omnibox commands.

**Parameters:**
- `extensionId` (string): The extension ID
- `keyword` (string): The keyword (e.g., "ext")

#### `omnibox_set_suggestions`

Sets suggestions for user input.

**Parameters:**
- `extensionId` (string): The extension ID
- `suggestions` (array): Array of suggestions
  - `content` (string): Suggestion content
  - `description` (string): Suggestion description
  - `type` (string): 'default' | 'url' | 'search'
  - `deletable` (boolean): Whether deletable

#### `omnibox_get_suggestions`

Gets current suggestions for an extension.

**Parameters:**
- `extensionId` (string): The extension ID

**Returns:** `OmniboxSuggestion[]`

#### `omnibox_clear_suggestions`

Clears all suggestions for an extension.

**Parameters:**
- `extensionId` (string): The extension ID

#### `omnibox_generate_suggestion_id`

Generates a unique suggestion ID.

**Returns:** `string`

---

## Identity API

### Overview

The Identity API allows extensions to authenticate users using OAuth and manage authentication tokens.

### Methods

#### `identity_get_token`

Gets an authentication token for a user.

**Parameters:**
- `accountId` (string): The account ID
- `scopes` (array): OAuth scopes

**Returns:** `TokenInfo`

#### `identity_remove_cached_token`

Removes a cached authentication token.

**Parameters:**
- `accountId` (string): The account ID

#### `identity_clear_all_cached_tokens`

Clears all cached authentication tokens.

#### `identity_launch_web_auth_flow`

Launches a web authentication flow.

**Parameters:**
- `authUrl` (string): The authentication URL
- `interactive` (boolean): Whether to show interactive UI

**Returns:** `WebAuthFlowResult`

#### `identity_get_redirect_url`

Gets the OAuth redirect URL.

**Returns:** `string`

#### `identity_get_profile_info`

Gets profile information for an account.

**Parameters:**
- `accountId` (string): The account ID

**Returns:** Profile information object

#### `identity_set_oauth_config`

Sets OAuth configuration.

**Parameters:**
- `config` (object):
  - `clientId` (string): OAuth client ID
  - `clientSecret` (string, optional): OAuth client secret
  - `scopes` (array): OAuth scopes
  - `redirectUrl` (string): OAuth redirect URL

---

## Top Sites API

### Overview

The Top Sites API allows extensions to access the user's most visited websites.

### Methods

#### `top_sites_get`

Gets the user's top sites.

**Returns:** `TopSite[]`

#### `top_sites_get_with_limit`

Gets top sites with a limit.

**Parameters:**
- `limit` (number): Maximum number of sites to return

**Returns:** `TopSite[]`

#### `top_sites_record_visit`

Records a site visit.

**Parameters:**
- `url` (string): The URL visited
- `title` (string): The page title

#### `top_sites_clear_all`

Clears all top sites data.

#### `top_sites_add`

Manually adds a top site.

**Parameters:**
- `site` (object):
  - `url` (string): Site URL
  - `title` (string): Site title
  - `faviconUrl` (string, optional): Favicon URL
  - `type` (string): Site type

#### `top_sites_remove`

Removes a top site.

**Parameters:**
- `url` (string): The URL to remove

---

## DevTools API

### Overview

The DevTools API allows extensions to interact with the browser's developer tools.

### Methods

#### `devtools_create_panel`

Creates a new DevTools panel.

**Parameters:**
- `extensionId` (string): The extension ID
- `title` (string): Panel title
- `panelType` (string): 'main' | 'sidebar'
- `iconPath` (string, optional): Icon path

**Returns:** `DevToolsPanel`

#### `devtools_get_panels`

Gets all panels for an extension.

**Parameters:**
- `extensionId` (string): The extension ID

**Returns:** `DevToolsPanel[]`

#### `devtools_remove_panel`

Removes a DevTools panel.

**Parameters:**
- `extensionId` (string): The extension ID
- `panelId` (string): The panel ID

#### `devtools_register_inspected_window`

Registers an inspected window.

**Parameters:**
- `window` (object): Window information
  - `tabId` (number): Tab ID
  - `windowId` (number): Window ID
  - `extensionId` (string): Extension ID

#### `devtools_get_inspected_window`

Gets inspected window information.

**Parameters:**
- `tabId` (number): Tab ID
- `extensionId` (string): Extension ID

**Returns:** `InspectedWindow`

#### `devtools_record_network_request`

Records a network request.

**Parameters:**
- `request` (object): Network request details

#### `devtools_record_network_response`

Records a network response.

**Parameters:**
- `response` (object): Network response details

#### `devtools_get_network_request`

Gets a network request by ID.

**Parameters:**
- `requestId` (string): Request ID

**Returns:** `NetworkRequest`

#### `devtools_get_network_response`

Gets a network response by ID.

**Parameters:**
- `requestId` (string): Request ID

**Returns:** `NetworkResponse`

#### `devtools_get_network_requests_for_tab`

Gets all network requests for a tab.

**Parameters:**
- `tabId` (number): Tab ID

**Returns:** `NetworkRequest[]`

---

## Site Isolation API

### Overview

The Site Isolation API provides information about the browser's site isolation system, which separates different websites into isolated processes for security.

### Methods

#### `get_or_create_site`

Gets or creates a site instance for a URL.

**Parameters:**
- `url` (string): The URL

**Returns:** `SiteInstance`

#### `release_site`

Releases a site instance.

**Parameters:**
- `url` (string): The URL

#### `get_isolation_policy`

Gets the current isolation policy.

**Returns:** `IsolationPolicy`

#### `set_isolation_policy`

Sets the isolation policy.

**Parameters:**
- `policy` (object): Isolation policy configuration

#### `get_site_instances`

Gets all active site instances.

**Returns:** `SiteInstance[]`

#### `get_processes`

Gets all isolated processes.

**Returns:** `ProcessInfo[]`

#### `get_process_for_url`

Gets the process for a specific URL.

**Parameters:**
- `url` (string): The URL

**Returns:** Process ID or null

#### `is_navigation_allowed`

Checks if navigation between URLs is allowed.

**Parameters:**
- `fromUrl` (string): Source URL
- `toUrl` (string): Destination URL

**Returns:** `boolean`

#### `get_isolation_stats`

Gets isolation statistics.

**Returns:** Statistics object

#### `cleanup_stale_data`

Cleans up stale site isolation data.

---

## Error Handling

### Error Types

All Extension APIs use a standardized error handling system with the following error types:

- `ExtensionNotFound`: Extension not found
- `PermissionDenied`: Permission denied
- `InvalidArgument`: Invalid argument provided
- `NotSupported`: Operation not supported
- `InternalError`: Internal error occurred
- `NetworkError`: Network error
- `Timeout`: Operation timed out
- `RateLimited`: Rate limit exceeded
- `InvalidState`: Invalid state

### Error Response Format

All errors are returned as strings with descriptive messages. The error type can be determined from the message content.

### Example Error Handling

```typescript
try {
  const result = await window.__EXODUS_TAURI_INVOKE__('context_menus_create', {
    extensionId: 'my-extension',
    createProperties: { /* ... */ }
  });
} catch (error) {
  console.error('Failed to create menu item:', error);
  // Handle error appropriately
}
```

---

## Performance Metrics

### Overview

The Extension API system includes built-in performance monitoring to track API call performance and identify bottlenecks.

### Metrics Collected

For each API call, the following metrics are collected:

- **API Name**: The API being called
- **Method Name**: The specific method
- **Extension ID**: The extension making the call
- **Duration**: Time taken in milliseconds
- **Success/Failure**: Whether the call succeeded
- **Timestamp**: When the call was made
- **Metadata**: Additional context information

### Aggregated Metrics

Metrics are aggregated to provide:

- Total calls per API
- Success/failure rates
- Average duration
- Min/max duration
- Total duration

### Using Metrics

Metrics can be accessed programmatically (not exposed to frontend by default) to monitor extension performance and identify issues.

---

## Security Considerations

### Site Isolation

Exodus Browser implements aerospace-grade site isolation:

- Each security origin runs in its own process
- Spectre/Meltdown mitigations enabled by default
- Strict same-origin policy enforcement
- Cross-site data leak prevention

### Permission Model

Extensions must explicitly request permissions:

- `contextMenus`: Add context menu items
- `webNavigation`: Monitor navigation events
- `sidePanel`: Create side panels
- `omnibox`: Add address bar suggestions
- `identity`: Authenticate users
- `topSites`: Access top sites
- `devtools`: Interact with DevTools

### Data Privacy

- All data processing happens locally
- No data leaves the user's machine
- Extensions are sandboxed
- Clear permission boundaries

---

## Best Practices

### 1. Error Handling

Always handle errors gracefully:

```typescript
try {
  await ContextMenusAPI.createMenuItem(extensionId, title, contexts);
} catch (error) {
  // Log error and show user-friendly message
  console.error('Failed to create menu item:', error);
}
```

### 2. Performance

- Batch operations when possible
- Use appropriate filters to reduce data transfer
- Cache results when appropriate
- Clean up resources when done

### 3. Security

- Validate all user input
- Use least privilege principle
- Don't expose sensitive data
- Follow same-origin policy

### 4. User Experience

- Provide clear feedback
- Handle long operations gracefully
- Respect user preferences
- Don't block the UI

---

## Migration from Chrome Extensions

### Compatibility

Exodus Browser aims for Chrome Extension Manifest V3 compatibility. Most Chrome extensions should work with minimal modifications.

### Key Differences

1. **Site Isolation**: Exodus has stricter site isolation by default
2. **Privacy**: All data is processed locally
3. **API Coverage**: Currently ~90% of Chrome APIs are supported
4. **Performance**: Optimized for local AI workloads

### Migration Steps

1. Test your extension in Exodus
2. Review permission requirements
3. Update any Chrome-specific APIs
4. Test site isolation behavior
5. Verify privacy compliance

---

## Support and Feedback

For issues, questions, or feedback:

- GitHub Issues: [Exodus Browser Repository]
- Documentation: [Exodus Docs]
- Community: [Exodus Community]

---

## Changelog

### Version 0.2.0 (2026-05-29)

- Added Context Menus API
- Added Web Navigation API
- Added Side Panel API
- Added Omnibox API
- Added Identity API
- Added Top Sites API
- Added DevTools API
- Enhanced Site Isolation API
- Added error handling system
- Added performance metrics
- Added comprehensive TypeScript types
- Added usage examples

---

*Last Updated: 2026-05-29*
