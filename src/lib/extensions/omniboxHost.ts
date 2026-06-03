/**
 * Exodus Browser — host integration for chrome.omnibox (keyword + events).
 * Aerospace-level error handling, security validation, and input validation.
 */

import { invoke, isTauri } from '@tauri-apps/api/core';

// Aerospace-level security validation patterns
const VALID_EXTENSION_ID_PATTERN = /^[a-zA-Z0-9_-]+$/;
const VALID_KEYWORD_PATTERN = /^[a-zA-Z0-9_-]+$/;
const VALID_TEXT_PATTERN = /^[a-zA-Z0-9_\s\-.,!?@#$%^&*()+=\[\]{};:'"<>/\\|`~]+$/;

/**
 * Aerospace-level validation for extension ID format.
 */
function validateExtensionId(extensionId: string): boolean {
  if (!extensionId || typeof extensionId !== 'string') {
    console.error('[OmniboxHost] Invalid extension ID');
    return false;
  }
  return VALID_EXTENSION_ID_PATTERN.test(extensionId);
}

/**
 * Aerospace-level validation for keyword format.
 */
function validateKeyword(keyword: string): boolean {
  if (!keyword || typeof keyword !== 'string') {
    console.error('[OmniboxHost] Invalid keyword');
    return false;
  }
  return VALID_KEYWORD_PATTERN.test(keyword);
}

/**
 * Aerospace-level validation for omnibox text format.
 */
function validateOmniboxText(text: string): boolean {
  if (!text || typeof text !== 'string') {
    console.error('[OmniboxHost] Invalid omnibox text');
    return false;
  }
  return VALID_TEXT_PATTERN.test(text);
}

/**
 * Aerospace-level validation for omnibox keyword entry structure.
 */
function validateOmniboxKeyword(entry: ExtensionOmniboxKeyword): boolean {
  if (!entry || typeof entry !== 'object') {
    console.error('[OmniboxHost] Invalid omnibox keyword entry');
    return false;
  }
  if (!validateExtensionId(entry.extensionId)) {
    console.error('[OmniboxHost] Invalid extensionId in omnibox keyword:', entry.extensionId);
    return false;
  }
  if (!validateKeyword(entry.keyword)) {
    console.error('[OmniboxHost] Invalid keyword in omnibox keyword:', entry.keyword);
    return false;
  }
  if (!entry.extensionName || typeof entry.extensionName !== 'string') {
    console.error('[OmniboxHost] Invalid extensionName in omnibox keyword');
    return false;
  }
  return true;
}

/** Extension omnibox keyword from manifest. */
export type ExtensionOmniboxKeyword = {
  extensionId: string;
  extensionName: string;
  keyword: string;
};

/** List enabled extensions with omnibox keywords. */
export async function listExtensionOmniboxKeywords(): Promise<ExtensionOmniboxKeyword[]> {
  if (!isTauri()) return [];
  try {
    const keywords = await invoke<ExtensionOmniboxKeyword[]>('extension_omnibox_list_keywords');
    
    // Aerospace-level validation of response
    if (!Array.isArray(keywords)) {
      console.error('[OmniboxHost] Invalid keywords response from backend');
      return [];
    }
    
    // Validate each keyword entry
    const validKeywords: ExtensionOmniboxKeyword[] = [];
    for (const keyword of keywords) {
      if (validateOmniboxKeyword(keyword)) {
        validKeywords.push(keyword);
      } else {
        console.error('[OmniboxHost] Skipping invalid omnibox keyword entry');
      }
    }
    
    return validKeywords;
  } catch (error) {
    console.error('extension_omnibox_list_keywords failed:', error);
    return [];
  }
}

/** Dispatch omnibox event to extension background (`onInputChanged`, `onInputEntered`, …). */
export async function dispatchExtensionOmniboxEvent(
  extensionId: string,
  event: 'onInputStarted' | 'onInputChanged' | 'onInputEntered' | 'onInputCancelled',
  text: string,
): Promise<void> {
  if (!isTauri()) return;
  
  // Aerospace-level input validation
  if (!validateExtensionId(extensionId)) {
    console.error('[OmniboxHost] Invalid extensionId for dispatchExtensionOmniboxEvent:', extensionId);
    return;
  }
  
  if (!event || typeof event !== 'string') {
    console.error('[OmniboxHost] Invalid event type for dispatchExtensionOmniboxEvent');
    return;
  }
  
  if (!validateOmniboxText(text)) {
    console.error('[OmniboxHost] Invalid text for dispatchExtensionOmniboxEvent:', text);
    return;
  }
  
  try {
    await invoke('extension_omnibox_dispatch', { extensionId, event, text });
  } catch (error) {
    console.error('extension_omnibox_dispatch failed:', error);
  }
}

/**
 * Match address bar input against extension omnibox keywords.
 * Returns keyword match and text after keyword (Chrome-style `keyword query`).
 * Aerospace-level input validation with graceful fallbacks.
 */
export function matchExtensionOmniboxKeyword(
  input: string,
  keywords: ExtensionOmniboxKeyword[],
): { entry: ExtensionOmniboxKeyword; query: string } | null {
  // Aerospace-level input validation
  if (!input || typeof input !== 'string') {
    console.error('[OmniboxHost] Invalid input for matchExtensionOmniboxKeyword');
    return null;
  }
  
  if (!Array.isArray(keywords)) {
    console.error('[OmniboxHost] Invalid keywords array for matchExtensionOmniboxKeyword');
    return null;
  }
  
  const trimmed = input.trim();
  if (!trimmed) return null;
  
  for (const entry of keywords) {
    // Validate each keyword entry before processing
    if (!validateOmniboxKeyword(entry)) {
      console.error('[OmniboxHost] Skipping invalid keyword entry in match');
      continue;
    }
    
    const prefix = `${entry.keyword} `;
    if (trimmed.startsWith(prefix)) {
      return { entry, query: trimmed.slice(prefix.length) };
    }
    if (trimmed === entry.keyword) {
      return { entry, query: '' };
    }
  }
  return null;
}
