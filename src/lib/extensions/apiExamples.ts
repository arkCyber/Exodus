/**
 * Exodus Browser — Extension API Usage Examples
 * 
 * This file provides examples of how to use the newly implemented Extension APIs
 * from the frontend TypeScript code.
 */

import type {
  ContextMenuItem,
  ContextType,
  NavigationEvent,
  SidePanelState,
  OmniboxSuggestion,
  TokenInfo,
  TopSite,
  DevToolsPanel,
  SiteInstance,
  IsolationPolicy,
} from './types';

/**
 * Context Menus API Examples
 */
export class ContextMenusAPI {
  /**
   * Create a context menu item
   */
  static async createMenuItem(extensionId: string, title: string, contexts: ContextType[]) {
    const result = await window.__EXODUS_TAURI_INVOKE__('context_menus_create', {
      extensionId,
      createProperties: {
        title,
        item_type: 'normal',
        enabled: true,
        checked: false,
        contexts,
      },
    });
    return result as ContextMenuItem;
  }

  /**
   * Update a context menu item
   */
  static async updateMenuItem(itemId: string, title?: string, enabled?: boolean) {
    const result = await window.__EXODUS_TAURI_INVOKE__('context_menus_update', {
      itemId,
      updateProperties: {
        title,
        enabled,
      },
    });
    return result as ContextMenuItem;
  }

  /**
   * Remove a context menu item
   */
  static async removeMenuItem(itemId: string) {
    await window.__EXODUS_TAURI_INVOKE__('context_menus_remove', { itemId });
  }

  /**
   * Get all menu items for an extension
   */
  static async getMenuItems(extensionId: string) {
    const result = await window.__EXODUS_TAURI_INVOKE__('context_menus_get_all', {
      extensionId,
    });
    return result as ContextMenuItem[];
  }

  /**
   * Get menu items for a specific context
   */
  static async getMenuItemsForContext(context: ContextType) {
    const result = await window.__EXODUS_TAURI_INVOKE__('context_menus_get_for_context', {
      context,
    });
    return result as ContextMenuItem[];
  }
}

/**
 * Web Navigation API Examples
 */
export class WebNavigationAPI {
  /**
   * Add a listener for navigation events
   */
  static async addListener(extensionId: string, eventTypes: string[]) {
    await window.__EXODUS_TAURI_INVOKE__('web_navigation_add_listener', {
      extensionId,
      eventTypes,
    });
  }

  /**
   * Remove a listener
   */
  static async removeListener(extensionId: string) {
    await window.__EXODUS_TAURI_INVOKE__('web_navigation_remove_listener', {
      extensionId,
    });
  }

  /**
   * Get frame information
   */
  static async getFrame(tabId: number, frameId: number) {
    const result = await window.__EXODUS_TAURI_INVOKE__('web_navigation_get_frame', {
      tabId,
      frameId,
    });
    return result;
  }

  /**
   * Get all frames for a tab
   */
  static async getAllFrames(tabId: number) {
    const result = await window.__EXODUS_TAURI_INVOKE__('web_navigation_get_all_frames', {
      tabId,
    });
    return result;
  }

  /**
   * Get navigation history
   */
  static async getHistory(filter?: any) {
    const result = await window.__EXODUS_TAURI_INVOKE__('web_navigation_get_history', {
      filter,
    });
    return result as NavigationEvent[];
  }
}

/**
 * Side Panel API Examples
 */
export class SidePanelAPI {
  /**
   * Set side panel options
   */
  static async setOptions(extensionId: string, path: string, title: string) {
    const result = await window.__EXODUS_TAURI_INVOKE__('side_panel_set_options', {
      extensionId,
      options: {
        path,
        title,
        behavior: 'openPanel',
      },
    });
    return result;
  }

  /**
   * Open side panel
   */
  static async openPanel(extensionId: string) {
    const result = await window.__EXODUS_TAURI_INVOKE__('side_panel_open', {
      extensionId,
    });
    return result as SidePanelState;
  }

  /**
   * Close side panel
   */
  static async closePanel(extensionId: string) {
    const result = await window.__EXODUS_TAURI_INVOKE__('side_panel_close', {
      extensionId,
    });
    return result as SidePanelState;
  }

  /**
   * Get panel state
   */
  static async getPanelState(extensionId: string) {
    const result = await window.__EXODUS_TAURI_INVOKE__('side_panel_get_state', {
      extensionId,
    });
    return result as SidePanelState;
  }

  /**
   * Enable panel
   */
  static async enablePanel(extensionId: string) {
    await window.__EXODUS_TAURI_INVOKE__('side_panel_enable', { extensionId });
  }

  /**
   * Disable panel
   */
  static async disablePanel(extensionId: string) {
    await window.__EXODUS_TAURI_INVOKE__('side_panel_disable', { extensionId });
  }
}

/**
 * Omnibox API Examples
 */
export class OmniboxAPI {
  /**
   * Set default suggestion
   */
  static async setDefaultSuggestion(extensionId: string, description: string) {
    await window.__EXODUS_TAURI_INVOKE__('omnibox_set_default_suggestion', {
      extensionId,
      suggestion: { description },
    });
  }

  /**
   * Set keyword
   */
  static async setKeyword(extensionId: string, keyword: string) {
    await window.__EXODUS_TAURI_INVOKE__('omnibox_set_keyword', {
      extensionId,
      keyword,
    });
  }

  /**
   * Set suggestions
   */
  static async setSuggestions(extensionId: string, suggestions: OmniboxSuggestion[]) {
    await window.__EXODUS_TAURI_INVOKE__('omnibox_set_suggestions', {
      extensionId,
      suggestions,
    });
  }

  /**
   * Get suggestions
   */
  static async getSuggestions(extensionId: string) {
    const result = await window.__EXODUS_TAURI_INVOKE__('omnibox_get_suggestions', {
      extensionId,
    });
    return result as OmniboxSuggestion[];
  }

  /**
   * Clear suggestions
   */
  static async clearSuggestions(extensionId: string) {
    await window.__EXODUS_TAURI_INVOKE__('omnibox_clear_suggestions', {
      extensionId,
    });
  }

  /**
   * Generate suggestion ID
   */
  static async generateSuggestionId() {
    const result = await window.__EXODUS_TAURI_INVOKE__('omnibox_generate_suggestion_id', {});
    return result as string;
  }
}

/**
 * Identity API Examples
 */
export class IdentityAPI {
  /**
   * Get authentication token
   */
  static async getToken(accountId: string, scopes: string[]) {
    const result = await window.__EXODUS_TAURI_INVOKE__('identity_get_token', {
      accountId,
      scopes,
    });
    return result as TokenInfo;
  }

  /**
   * Remove cached token
   */
  static async removeCachedToken(accountId: string) {
    await window.__EXODUS_TAURI_INVOKE__('identity_remove_cached_token', {
      accountId,
    });
  }

  /**
   * Clear all cached tokens
   */
  static async clearAllCachedTokens() {
    await window.__EXODUS_TAURI_INVOKE__('identity_clear_all_cached_tokens', {});
  }

  /**
   * Launch web auth flow
   */
  static async launchWebAuthFlow(authUrl: string, interactive: boolean) {
    const result = await window.__EXODUS_TAURI_INVOKE__('identity_launch_web_auth_flow', {
      authUrl,
      interactive,
    });
    return result;
  }

  /**
   * Get redirect URL
   */
  static async getRedirectUrl() {
    const result = await window.__EXODUS_TAURI_INVOKE__('identity_get_redirect_url', {});
    return result as string;
  }

  /**
   * Get profile info
   */
  static async getProfileInfo(accountId: string) {
    const result = await window.__EXODUS_TAURI_INVOKE__('identity_get_profile_info', {
      accountId,
    });
    return result;
  }

  /**
   * Set OAuth configuration
   */
  static async setOAuthConfig(config: any) {
    await window.__EXODUS_TAURI_INVOKE__('identity_set_oauth_config', { config });
  }
}

/**
 * Top Sites API Examples
 */
export class TopSitesAPI {
  /**
   * Get top sites
   */
  static async getTopSites() {
    const result = await window.__EXODUS_TAURI_INVOKE__('top_sites_get', {});
    return result as TopSite[];
  }

  /**
   * Get top sites with limit
   */
  static async getTopSitesWithLimit(limit: number) {
    const result = await window.__EXODUS_TAURI_INVOKE__('top_sites_get_with_limit', {
      limit,
    });
    return result as TopSite[];
  }

  /**
   * Record a site visit
   */
  static async recordVisit(url: string, title: string) {
    await window.__EXODUS_TAURI_INVOKE__('top_sites_record_visit', {
      url,
      title,
    });
  }

  /**
   * Clear all top sites
   */
  static async clearAll() {
    await window.__EXODUS_TAURI_INVOKE__('top_sites_clear_all', {});
  }

  /**
   * Add a top site manually
   */
  static async addTopSite(site: TopSite) {
    await window.__EXODUS_TAURI_INVOKE__('top_sites_add', { site });
  }

  /**
   * Remove a top site
   */
  static async removeTopSite(url: string) {
    await window.__EXODUS_TAURI_INVOKE__('top_sites_remove', { url });
  }
}

/**
 * DevTools API Examples
 */
export class DevToolsAPI {
  /**
   * Create a DevTools panel
   */
  static async createPanel(extensionId: string, title: string, panelType: string, iconPath?: string) {
    const result = await window.__EXODUS_TAURI_INVOKE__('devtools_create_panel', {
      extensionId,
      title,
      panelType,
      iconPath,
    });
    return result as DevToolsPanel;
  }

  /**
   * Get panels for an extension
   */
  static async getPanels(extensionId: string) {
    const result = await window.__EXODUS_TAURI_INVOKE__('devtools_get_panels', {
      extensionId,
    });
    return result as DevToolsPanel[];
  }

  /**
   * Remove a panel
   */
  static async removePanel(extensionId: string, panelId: string) {
    await window.__EXODUS_TAURI_INVOKE__('devtools_remove_panel', {
      extensionId,
      panelId,
    });
  }

  /**
   * Register inspected window
   */
  static async registerInspectedWindow(window: any) {
    await window.__EXODUS_TAURI_INVOKE__('devtools_register_inspected_window', {
      window,
    });
  }

  /**
   * Get inspected window
   */
  static async getInspectedWindow(tabId: number, extensionId: string) {
    const result = await window.__EXODUS_TAURI_INVOKE__('devtools_get_inspected_window', {
      tabId,
      extensionId,
    });
    return result;
  }

  /**
   * Record network request
   */
  static async recordNetworkRequest(request: any) {
    await window.__EXODUS_TAURI_INVOKE__('devtools_record_network_request', {
      request,
    });
  }

  /**
   * Record network response
   */
  static async recordNetworkResponse(response: any) {
    await window.__EXODUS_TAURI_INVOKE__('devtools_record_network_response', {
      response,
    });
  }

  /**
   * Get network request
   */
  static async getNetworkRequest(requestId: string) {
    const result = await window.__EXODUS_TAURI_INVOKE__('devtools_get_network_request', {
      requestId,
    });
    return result;
  }

  /**
   * Get network response
   */
  static async getNetworkResponse(requestId: string) {
    const result = await window.__EXODUS_TAURI_INVOKE__('devtools_get_network_response', {
      requestId,
    });
    return result;
  }

  /**
   * Get network requests for tab
   */
  static async getNetworkRequestsForTab(tabId: number) {
    const result = await window.__EXODUS_TAURI_INVOKE__('devtools_get_network_requests_for_tab', {
      tabId,
    });
    return result;
  }
}

/**
 * Site Isolation API Examples
 */
export class SiteIsolationAPI {
  /**
   * Get or create a site instance
   */
  static async getOrCreateSite(url: string) {
    const result = await window.__EXODUS_TAURI_INVOKE__('get_or_create_site', { url });
    return result as SiteInstance;
  }

  /**
   * Release a site instance
   */
  static async releaseSite(url: string) {
    await window.__EXODUS_TAURI_INVOKE__('release_site', { url });
  }

  /**
   * Get isolation policy
   */
  static async getIsolationPolicy() {
    const result = await window.__EXODUS_TAURI_INVOKE__('get_isolation_policy', {});
    return result as IsolationPolicy;
  }

  /**
   * Set isolation policy
   */
  static async setIsolationPolicy(policy: IsolationPolicy) {
    await window.__EXODUS_TAURI_INVOKE__('set_isolation_policy', { policy });
  }

  /**
   * Get all site instances
   */
  static async getSiteInstances() {
    const result = await window.__EXODUS_TAURI_INVOKE__('get_site_instances', {});
    return result as SiteInstance[];
  }

  /**
   * Get all processes
   */
  static async getProcesses() {
    const result = await window.__EXODUS_TAURI_INVOKE__('get_processes', {});
    return result;
  }

  /**
   * Get process for a URL
   */
  static async getProcessForUrl(url: string) {
    const result = await window.__EXODUS_TAURI_INVOKE__('get_process_for_url', { url });
    return result;
  }

  /**
   * Check if navigation is allowed
   */
  static async isNavigationAllowed(fromUrl: string, toUrl: string) {
    const result = await window.__EXODUS_TAURI_INVOKE__('is_navigation_allowed', {
      fromUrl,
      toUrl,
    });
    return result as boolean;
  }

  /**
   * Get isolation statistics
   */
  static async getIsolationStats() {
    const result = await window.__EXODUS_TAURI_INVOKE__('get_isolation_stats', {});
    return result;
  }

  /**
   * Clean up stale data
   */
  static async cleanupStaleData() {
    await window.__EXODUS_TAURI_INVOKE__('cleanup_stale_data', {});
  }
}

/**
 * Comprehensive example: Using multiple APIs together
 */
export async function comprehensiveExtensionExample() {
  const extensionId = 'my-extension';

  // 1. Set up context menu
  await ContextMenusAPI.createMenuItem(
    extensionId,
    'My Extension Action',
    ['all']
  );

  // 2. Set up side panel
  await SidePanelAPI.setOptions(
    extensionId,
    'panel.html',
    'My Panel'
  );

  // 3. Set up omnibox
  await OmniboxAPI.setKeyword(extensionId, 'myext');
  await OmniboxAPI.setDefaultSuggestion(
    extensionId,
    'Search with My Extension'
  );

  // 4. Monitor navigation
  await WebNavigationAPI.addListener(
    extensionId,
    ['beforeNavigate', 'completed']
  );

  // 5. Get top sites
  const topSites = await TopSitesAPI.getTopSites();
  console.log('Top sites:', topSites);

  // 6. Check site isolation
  const policy = await SiteIsolationAPI.getIsolationPolicy();
  console.log('Isolation policy:', policy);
}
