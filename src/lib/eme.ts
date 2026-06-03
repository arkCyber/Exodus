/**
 * Exodus Browser — Encrypted Media Extensions (EME) API
 * 
 * Provides the JavaScript API for encrypted media playback using Widevine DRM.
 * This implements the EME specification for MediaKeys, MediaKeySession, etc.
 */

import { invoke, isTauri } from '@tauri-apps/api/core';

/** DRM key system */
export const WIDEVINE_KEY_SYSTEM = 'com.widevine.alpha';

/** DRM session state */
export enum DrmSessionState {
  Idle = 'idle',
  Created = 'created',
  KeyRequest = 'keyRequest',
  KeyReady = 'keyReady',
  KeyError = 'keyError',
  Closed = 'closed',
}

/** DRM key request */
export interface DrmKeyRequest {
  sessionId: string;
  initDataType: string;
  initData: number[];
  keySystem: string;
}

/** DRM key response */
export interface DrmKeyResponse {
  sessionId: string;
  keyData: number[];
}

/** DRM session information */
export interface DrmSession {
  sessionId: string;
  keySystem: string;
  state: DrmSessionState;
  createdAt: number;
  mediaUrl?: string;
}

/** MediaKeySession event types */
export interface MediaKeySessionEventMap {
  message: CustomEvent<MediaKeyMessageEvent>;
  keystatuseschange: Event;
  keyadded: Event;
  keyerror: CustomEvent<MediaKeyErrorEvent>;
}

/** MediaKeyMessageEvent */
export interface MediaKeyMessageEvent {
  messageType: 'license-request' | 'license-renewal' | 'license-release' | 'individualization-request';
  message: ArrayBuffer;
}

/** MediaKeyErrorEvent */
export interface MediaKeyErrorEvent {
  errorCode: number;
  systemCode: number;
}

/**
 * MediaKeySession - Represents a session for key exchange with a CDM
 */
export class MediaKeySession extends EventTarget {
  private sessionId: string | null = null;
  private closed = false;
  private keySystem: string;
  private onMessageCallback: ((event: MediaKeyMessageEvent) => void) | null = null;
  private onKeyStatusesChangeCallback: (() => void) | null = null;

  constructor(keySystem: string) {
    super();
    this.keySystem = keySystem;
  }

  /**
   * Generate a key request
   */
  async generateRequest(initDataType: string, initData: ArrayBuffer): Promise<void> {
    if (this.closed) {
      throw new DOMException('Session is closed', 'InvalidStateError');
    }

    if (!isTauri()) {
      console.warn('[EME] Not in Tauri environment, skipping key request');
      return;
    }

    try {
      const initDataArray = Array.from(new Uint8Array(initData));
      const request = await invoke<DrmKeyRequest>('drm_generate_key_request', {
        sessionId: this.sessionId || '',
      });

      // Dispatch message event with the key request
      const event = new CustomEvent<MediaKeyMessageEvent>('message', {
        detail: {
          messageType: 'license-request',
          message: new Uint8Array(request.initData).buffer,
        },
      });

      this.dispatchEvent(event);
    } catch (error) {
      console.error('[EME] Failed to generate key request:', error);
      throw error;
    }
  }

  /**
   * Provide a license or other key data to the session
   */
  async update(response: ArrayBuffer): Promise<void> {
    if (this.closed) {
      throw new DOMException('Session is closed', 'InvalidStateError');
    }

    if (!isTauri()) {
      console.warn('[EME] Not in Tauri environment, skipping key update');
      return;
    }

    try {
      const keyData = Array.from(new Uint8Array(response));
      const keyResponse: DrmKeyResponse = {
        sessionId: this.sessionId || '',
        keyData,
      };

      await invoke('drm_process_key_response', {
        response: keyResponse,
      });

      // Dispatch keystatuseschange event
      this.dispatchEvent(new Event('keystatuseschange'));
    } catch (error) {
      console.error('[EME] Failed to update key:', error);
      throw error;
    }
  }

  /**
   * Close the session and release resources
   */
  async close(): Promise<void> {
    if (this.closed) {
      return;
    }

    this.closed = true;

    if (!isTauri()) {
      console.warn('[EME] Not in Tauri environment, skipping session close');
      return;
    }

    try {
      if (this.sessionId) {
        await invoke('drm_close_session', {
          sessionId: this.sessionId,
        });
      }
    } catch (error) {
      console.error('[EME] Failed to close session:', error);
    }
  }

  /**
   * Remove the session and associated keys
   */
  async remove(): Promise<void> {
    await this.close();
  }

  /**
   * Get the session ID
   */
  get sessionId(): string | null {
    return this.sessionId;
  }

  /**
   * Get the expiration time of the session
   */
  get expiration(): number | null {
    // Not implemented in current backend
    return null;
  }

  /**
   * Get the key statuses for the session
   */
  getKeyStatuses(): MediaKeyStatusMap {
    // Simplified implementation
    return new Map([
      ['key-id', 'usable'],
    ]);
  }

  /**
   * Set the onmessage event handler
   */
  set onmessage(callback: ((event: MediaKeyMessageEvent) => void) | null) {
    this.onMessageCallback = callback;
    if (callback) {
      this.addEventListener('message', (e: Event) => {
        const customEvent = e as CustomEvent<MediaKeyMessageEvent>;
        callback(customEvent.detail);
      });
    } else {
      this.removeEventListener('message', () => {});
    }
  }

  /**
   * Set the onkeystatuseschange event handler
   */
  set onkeystatuseschange(callback: (() => void) | null) {
    this.onKeyStatusesChangeCallback = callback;
    if (callback) {
      this.addEventListener('keystatuseschange', callback);
    } else {
      this.removeEventListener('keystatuseschange', () => {});
    }
  }
}

/**
 * MediaKeys - Represents a set of keys for a key system
 */
export class MediaKeys {
  private keySystem: string;

  constructor(keySystem: string) {
    this.keySystem = keySystem;
  }

  /**
   * Create a new MediaKeySession
   */
  createSession(sessionType: string = 'temporary'): MediaKeySession {
    if (sessionType !== 'temporary' && sessionType !== 'persistent-license') {
      throw new DOMException('Invalid session type', 'NotSupportedError');
    }

    return new MediaKeySession(this.keySystem);
  }

  /**
   * Set the server certificate for the key system
   */
  async setServerCertificate(serverCertificate: ArrayBuffer): Promise<boolean> {
    // Not implemented in current backend
    console.warn('[EME] setServerCertificate not implemented');
    return false;
  }
}

/**
 * MediaKeyStatusMap - Map of key IDs to their status
 */
export type MediaKeyStatusMap = Map<string, MediaKeyStatus>;

/**
 * MediaKeyStatus - Status of a media key
 */
export type MediaKeyStatus =
  | 'usable'
  | 'expired'
  | 'released'
  | 'output-restricted'
  | 'output-downscaled'
  | 'status-pending'
  | 'internal-error';

/**
 * Check if a key system is supported
 */
export function isTypeSupported(keySystem: string, initDataType?: string): boolean {
  if (keySystem !== WIDEVINE_KEY_SYSTEM) {
    return false;
  }

  if (initDataType && initDataType !== 'cenc' && initDataType !== 'keyids') {
    return false;
  }

  return true;
}

/**
 * Request access to a key system
 */
export async function requestMediaKeySystemAccess(
  keySystem: string,
  supportedConfigurations: MediaKeySystemConfiguration[]
): Promise<MediaKeys> {
  if (!isTypeSupported(keySystem)) {
    throw new DOMException('Key system not supported', 'NotSupportedError');
  }

  // Check if Widevine CDM is available
  if (isTauri()) {
    try {
      const available = await invoke<boolean>('drm_is_cdm_available');
      if (!available) {
        throw new DOMException('CDM not available', 'NotSupportedError');
      }
    } catch (error) {
      console.error('[EME] Failed to check CDM availability:', error);
      throw new DOMException('Failed to check CDM availability', 'NotSupportedError');
    }
  }

  return new MediaKeys(keySystem);
}

/**
 * MediaKeySystemConfiguration - Configuration for a key system
 */
export interface MediaKeySystemConfiguration {
  initDataTypes?: string[];
  audioCapabilities?: MediaKeySystemMediaCapability[];
  videoCapabilities?: MediaKeySystemMediaCapability[];
  distinctiveIdentifier?: 'required' | 'optional' | 'not-allowed';
  persistentState?: 'required' | 'optional' | 'not-allowed';
  sessionTypes?: string[];
}

/**
 * MediaKeySystemMediaCapability - Media capability for a key system
 */
export interface MediaKeySystemMediaCapability {
  contentType: string;
  robustness?: string;
}

/**
 * Set the license server URL for Widevine DRM
 */
export async function setLicenseServerUrl(url: string): Promise<void> {
  if (!isTauri()) {
    console.warn('[EME] Not in Tauri environment, skipping license server URL set');
    return;
  }

  try {
    await invoke('drm_set_license_server_url', { url });
  } catch (error) {
    console.error('[EME] Failed to set license server URL:', error);
    throw error;
  }
}

/**
 * Get the license server URL for Widevine DRM
 */
export async function getLicenseServerUrl(): Promise<string | null> {
  if (!isTauri()) {
    console.warn('[EME] Not in Tauri environment, returning null for license server URL');
    return null;
  }

  try {
    return await invoke<string | null>('drm_get_license_server_url');
  } catch (error) {
    console.error('[EME] Failed to get license server URL:', error);
    return null;
  }
}

/**
 * Acquire a license from the license server
 */
export async function acquireLicense(
  sessionId: string,
  challenge: ArrayBuffer
): Promise<ArrayBuffer> {
  if (!isTauri()) {
    console.warn('[EME] Not in Tauri environment, returning empty license');
    return new ArrayBuffer(0);
  }

  try {
    const challengeArray = Array.from(new Uint8Array(challenge));
    const licenseData = await invoke<number[]>('drm_acquire_license', {
      sessionId,
      challenge: challengeArray,
    });
    return new Uint8Array(licenseData).buffer;
  } catch (error) {
    console.error('[EME] Failed to acquire license:', error);
    throw error;
  }
}

/**
 * Check if Widevine DRM is enabled
 */
export async function isDrmEnabled(): Promise<boolean> {
  if (!isTauri()) {
    return false;
  }

  try {
    return await invoke<boolean>('drm_is_enabled');
  } catch (error) {
    console.error('[EME] Failed to check DRM status:', error);
    return false;
  }
}

/**
 * Enable Widevine DRM
 */
export async function enableDrm(): Promise<void> {
  if (!isTauri()) {
    console.warn('[EME] Not in Tauri environment, skipping DRM enable');
    return;
  }

  try {
    await invoke('drm_enable');
  } catch (error) {
    console.error('[EME] Failed to enable DRM:', error);
    throw error;
  }
}

/**
 * Disable Widevine DRM
 */
export async function disableDrm(): Promise<void> {
  if (!isTauri()) {
    console.warn('[EME] Not in Tauri environment, skipping DRM disable');
    return;
  }

  try {
    await invoke('drm_disable');
  } catch (error) {
    console.error('[EME] Failed to disable DRM:', error);
    throw error;
  }
}
