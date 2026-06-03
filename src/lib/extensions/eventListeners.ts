/**
 * Exodus Browser — Extension API Event Listeners
 * 
 * This module provides event listeners for Extension API events,
 * allowing extensions to react to browser events and state changes.
 */

import type {
  MenuClickEvent,
  NavigationEvent,
  OmniboxInputChangedEvent,
  OmniboxSuggestionEnteredEvent,
} from './types';

/**
 * Event listener handler type
 */
export type EventHandler<T = any> = (event: T) => void | Promise<void>;

/**
 * Event listener registry
 */
class EventListenerRegistry {
  private listeners: Map<string, Set<EventHandler>> = new Map();

  /**
   * Register an event listener
   */
  on(event: string, handler: EventHandler): void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event)!.add(handler);
  }

  /**
   * Unregister an event listener
   */
  off(event: string, handler: EventHandler): void {
    const handlers = this.listeners.get(event);
    if (handlers) {
      handlers.delete(handler);
      if (handlers.size === 0) {
        this.listeners.delete(event);
      }
    }
  }

  /**
   * Emit an event to all listeners
   */
  async emit(event: string, data: any): Promise<void> {
    const handlers = this.listeners.get(event);
    if (handlers) {
      const promises = Array.from(handlers).map(handler => {
        try {
          return Promise.resolve(handler(data));
        } catch (error) {
          console.error(`Error in event handler for ${event}:`, error);
          return Promise.resolve();
        }
      });
      await Promise.all(promises);
    }
  }

  /**
   * Remove all listeners for an event
   */
  removeAllListeners(event?: string): void {
    if (event) {
      this.listeners.delete(event);
    } else {
      this.listeners.clear();
    }
  }

  /**
   * Get listener count for an event
   */
  listenerCount(event: string): number {
    return this.listeners.get(event)?.size || 0;
  }
}

// Global event registry
const eventRegistry = new EventListenerRegistry();

/**
 * Context Menu Event Listeners
 */
export class ContextMenuEvents {
  /**
   * Listen for context menu click events
   */
  static onClick(handler: EventHandler<MenuClickEvent>): () => void {
    eventRegistry.on('context-menu-clicked', handler);
    
    // Also listen to Tauri events
    const unlisten = window.__EXODUS_TAURI_LISTEN__('context-menu-clicked', (event: MenuClickEvent) => {
      eventRegistry.emit('context-menu-clicked', event);
    });

    // Return cleanup function
    return () => {
      eventRegistry.off('context-menu-clicked', handler);
      unlisten();
    };
  }
}

/**
 * Web Navigation Event Listeners
 */
export class WebNavigationEvents {
  /**
   * Listen for before navigate events
   */
  static onBeforeNavigate(handler: EventHandler<NavigationEvent>): () => void {
    eventRegistry.on('web-navigation-before-navigate', handler);
    
    const unlisten = window.__EXODUS_TAURI_LISTEN__('web-navigation-event', (event: any) => {
      if (event.eventType === 'beforeNavigate') {
        eventRegistry.emit('web-navigation-before-navigate', event);
      }
    });

    return () => {
      eventRegistry.off('web-navigation-before-navigate', handler);
      unlisten();
    };
  }

  /**
   * Listen for committed events
   */
  static onCommitted(handler: EventHandler<NavigationEvent>): () => void {
    eventRegistry.on('web-navigation-committed', handler);
    
    const unlisten = window.__EXODUS_TAURI_LISTEN__('web-navigation-event', (event: any) => {
      if (event.eventType === 'committed') {
        eventRegistry.emit('web-navigation-committed', event);
      }
    });

    return () => {
      eventRegistry.off('web-navigation-committed', handler);
      unlisten();
    };
  }

  /**
   * Listen for completed events
   */
  static onCompleted(handler: EventHandler<NavigationEvent>): () => void {
    eventRegistry.on('web-navigation-completed', handler);
    
    const unlisten = window.__EXODUS_TAURI_LISTEN__('web-navigation-event', (event: any) => {
      if (event.eventType === 'completed') {
        eventRegistry.emit('web-navigation-completed', event);
      }
    });

    return () => {
      eventRegistry.off('web-navigation-completed', handler);
      unlisten();
    };
  }

  /**
   * Listen for error occurred events
   */
  static onErrorOccurred(handler: EventHandler<NavigationEvent>): () => void {
    eventRegistry.on('web-navigation-error-occurred', handler);
    
    const unlisten = window.__EXODUS_TAURI_LISTEN__('web-navigation-event', (event: any) => {
      if (event.eventType === 'errorOccurred') {
        eventRegistry.emit('web-navigation-error-occurred', event);
      }
    });

    return () => {
      eventRegistry.off('web-navigation-error-occurred', handler);
      unlisten();
    };
  }
}

/**
 * Side Panel Event Listeners
 */
export class SidePanelEvents {
  /**
   * Listen for side panel opened events
   */
  static onOpened(handler: EventHandler<{ extensionId: string; state: any }>): () => void {
    eventRegistry.on('side-panel-opened', handler);
    
    const unlisten = window.__EXODUS_TAURI_LISTEN__('side-panel-opened', (event: any) => {
      eventRegistry.emit('side-panel-opened', event);
    });

    return () => {
      eventRegistry.off('side-panel-opened', handler);
      unlisten();
    };
  }

  /**
   * Listen for side panel closed events
   */
  static onClosed(handler: EventHandler<{ extensionId: string; state: any }>): () => void {
    eventRegistry.on('side-panel-closed', handler);
    
    const unlisten = window.__EXODUS_TAURI_LISTEN__('side-panel-closed', (event: any) => {
      eventRegistry.emit('side-panel-closed', event);
    });

    return () => {
      eventRegistry.off('side-panel-closed', handler);
      unlisten();
    };
  }
}

/**
 * Omnibox Event Listeners
 */
export class OmniboxEvents {
  /**
   * Listen for omnibox input changed events
   */
  static onInputChanged(handler: EventHandler<OmniboxInputChangedEvent>): () => void {
    eventRegistry.on('omnibox-input-changed', handler);
    
    const unlisten = window.__EXODUS_TAURI_LISTEN__('omnibox-input-changed', (event: OmniboxInputChangedEvent) => {
      eventRegistry.emit('omnibox-input-changed', event);
    });

    return () => {
      eventRegistry.off('omnibox-input-changed', handler);
      unlisten();
    };
  }

  /**
   * Listen for omnibox suggestion entered events
   */
  static onSuggestionEntered(handler: EventHandler<OmniboxSuggestionEnteredEvent>): () => void {
    eventRegistry.on('omnibox-suggestion-entered', handler);
    
    const unlisten = window.__EXODUS_TAURI_LISTEN__('omnibox-suggestion-entered', (event: OmniboxSuggestionEnteredEvent) => {
      eventRegistry.emit('omnibox-suggestion-entered', event);
    });

    return () => {
      eventRegistry.off('omnibox-suggestion-entered', handler);
      unlisten();
    };
  }
}

/**
 * Site Isolation Event Listeners
 */
export class SiteIsolationEvents {
  /**
   * Listen for process crash events
   */
  static onProcessCrash(handler: EventHandler<{ processId: string; siteId: string }>): () => void {
    eventRegistry.on('site-isolation-process-crash', handler);
    
    const unlisten = window.__EXODUS_TAURI_LISTEN__('site-isolation-process-crash', (event: any) => {
      eventRegistry.emit('site-isolation-process-crash', event);
    });

    return () => {
      eventRegistry.off('site-isolation-process-crash', handler);
      unlisten();
    };
  }

  /**
   * Listen for process recovery events
   */
  static onProcessRecovery(handler: EventHandler<{ processId: string; siteId: string }>): () => void {
    eventRegistry.on('site-isolation-process-recovery', handler);
    
    const unlisten = window.__EXODUS_TAURI_LISTEN__('site-isolation-process-recovery', (event: any) => {
      eventRegistry.emit('site-isolation-process-recovery', event);
    });

    return () => {
      eventRegistry.off('site-isolation-process-recovery', handler);
      unlisten();
    };
  }

  /**
   * Listen for site blacklisted events
   */
  static onSiteBlacklisted(handler: EventHandler<{ siteId: string }>): () => void {
    eventRegistry.on('site-isolation-site-blacklisted', handler);
    
    const unlisten = window.__EXODUS_TAURI_LISTEN__('site-isolation-site-blacklisted', (event: any) => {
      eventRegistry.emit('site-isolation-site-blacklisted', event);
    });

    return () => {
      eventRegistry.off('site-isolation-site-blacklisted', handler);
      unlisten();
    };
  }
}

/**
 * Extension Permission Event Listeners
 */
export class PermissionEvents {
  /**
   * Listen for permission request events
   */
  static onRequest(handler: EventHandler<{ extensionId: string; permission: string }>): () => void {
    eventRegistry.on('permission-request', handler);
    
    const unlisten = window.__EXODUS_TAURI_LISTEN__('permission-request', (event: any) => {
      eventRegistry.emit('permission-request', event);
    });

    return () => {
      eventRegistry.off('permission-request', handler);
      unlisten();
    };
  }

  /**
   * Listen for permission granted events
   */
  static onGranted(handler: EventHandler<{ extensionId: string; permission: string }>): () => void {
    eventRegistry.on('permission-granted', handler);
    
    const unlisten = window.__EXODUS_TAURI_LISTEN__('permission-granted', (event: any) => {
      eventRegistry.emit('permission-granted', event);
    });

    return () => {
      eventRegistry.off('permission-granted', handler);
      unlisten();
    };
  }

  /**
   * Listen for permission denied events
   */
  static onDenied(handler: EventHandler<{ extensionId: string; permission: string }>): () => void {
    eventRegistry.on('permission-denied', handler);
    
    const unlisten = window.__EXODUS_TAURI_LISTEN__('permission-denied', (event: any) => {
      eventRegistry.emit('permission-denied', event);
    });

    return () => {
      eventRegistry.off('permission-denied', handler);
      unlisten();
    };
  }
}

/**
 * Utility function to listen to all events
 */
export function listenToAllEvents() {
  console.log('Listening to all extension API events...');
  
  // Log all events for debugging
  eventRegistry.on('*', (event: any) => {
    console.log('Extension API Event:', event);
  });
}

/**
 * Utility function to stop listening to all events
 */
export function stopListeningToAllEvents() {
  eventRegistry.removeAllListeners();
  console.log('Stopped listening to all extension API events');
}

/**
 * Example: Using event listeners
 */
export function eventListenerExample() {
  const extensionId = 'my-extension';

  // Context menu click
  const contextMenuUnlisten = ContextMenuEvents.onClick((event) => {
    console.log('Context menu clicked:', event);
    if (event.extensionId === extensionId) {
      // Handle the click
    }
  });

  // Web navigation
  const navUnlisten = WebNavigationEvents.onCompleted((event) => {
    console.log('Navigation completed:', event);
  });

  // Side panel
  const panelUnlisten = SidePanelEvents.onOpened((event) => {
    console.log('Side panel opened:', event);
  });

  // Omnibox
  const omniboxUnlisten = OmniboxEvents.onInputChanged((event) => {
    console.log('Omnibox input changed:', event);
    // Update suggestions based on input
  });

  // Cleanup function
  return () => {
    contextMenuUnlisten();
    navUnlisten();
    panelUnlisten();
    omniboxUnlisten();
  };
}
