/**
 * Exodus Browser — Web Extension types (Manifest V3 subset).
 */

/** Store catalog entry from `extension_store_list`. */
export type StoreExtensionEntry = {
  id: string;
  name: string;
  version: string;
  description?: string;
  path: string;
  installed: boolean;
};

/** Tab create ack for `extension_tabs_create_ack`. */
export type TabCreateAck = {
  requestId: string;
  sourceWebviewLabel: string;
  chromeTabId: number;
  tabId: string;
  url: string;
  title: string;
};

/** Extension summary from `extension_list`. */
export type ExtensionInfo = {
  id: string;
  name: string;
  version: string;
  description?: string | null;
  enabled: boolean;
  permissions: string[];
  path: string;
  actionPopup?: string | null;
};
