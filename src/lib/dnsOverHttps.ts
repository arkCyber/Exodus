/**
 * DNS over HTTPS (DoH) API for Exodus Browser
 * Encrypts DNS queries to improve privacy and security
 */

import { invoke } from '@tauri-apps/api/core';

export interface DohProvider {
  name: string;
  url: string;
  enabled: boolean;
}

export interface DohSettings {
  enabled: boolean;
  providers: DohProvider[];
  fallback_to_system: boolean;
}

/**
 * Enable DNS over HTTPS
 */
export async function enableDoh(): Promise<void> {
  return invoke('enable_doh');
}

/**
 * Disable DNS over HTTPS
 */
export async function disableDoh(): Promise<void> {
  return invoke('disable_doh');
}

/**
 * Check if DNS over HTTPS is enabled
 */
export async function isDohEnabled(): Promise<boolean> {
  return invoke('is_doh_enabled');
}

/**
 * Set DNS over HTTPS provider
 */
export async function setDohProvider(providerName: string): Promise<void> {
  return invoke('set_doh_provider', { providerName });
}

/**
 * Get active DNS over HTTPS provider
 */
export async function getActiveDohProvider(): Promise<DohProvider | null> {
  return invoke('get_active_doh_provider');
}

/**
 * Add custom DNS over HTTPS provider
 */
export async function addDohProvider(name: string, url: string): Promise<void> {
  return invoke('add_doh_provider', { name, url });
}

/**
 * Remove DNS over HTTPS provider
 */
export async function removeDohProvider(name: string): Promise<void> {
  return invoke('remove_doh_provider', { name });
}

/**
 * Set fallback to system DNS
 */
export async function setDohFallback(fallback: boolean): Promise<void> {
  return invoke('set_doh_fallback', { fallback });
}

/**
 * Get all DNS over HTTPS providers
 */
export async function getDohProviders(): Promise<DohProvider[]> {
  return invoke('get_doh_providers');
}

/**
 * Get DNS over HTTPS settings
 */
export async function getDohSettings(): Promise<DohSettings> {
  return invoke('get_doh_settings');
}
