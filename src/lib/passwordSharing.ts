/**
 * Password Sharing API for Exodus Browser
 * Provides secure password sharing with trusted contacts
 */

import { invoke } from '@tauri-apps/api/core';

export interface TrustedContact {
  id: string;
  name: string;
  email: string;
  public_key: string;
  trusted_since: string;
}

export interface SharedPassword {
  id: string;
  title: string;
  url: string;
  username: string;
  encrypted_password: string;
  shared_with: string[];
  shared_at: string;
  expires_at: string | null;
}

export interface PasswordSharingSettings {
  enabled: boolean;
  require_approval: boolean;
  auto_expire_days: number;
}

/**
 * Enable password sharing
 */
export async function enablePasswordSharing(): Promise<void> {
  return invoke('enable_password_sharing');
}

/**
 * Disable password sharing
 */
export async function disablePasswordSharing(): Promise<void> {
  return invoke('disable_password_sharing');
}

/**
 * Check if password sharing is enabled
 */
export async function isPasswordSharingEnabled(): Promise<boolean> {
  return invoke('is_password_sharing_enabled');
}

/**
 * Add trusted contact
 */
export async function addTrustedContact(name: string, email: string, publicKey: string): Promise<string> {
  return invoke('add_trusted_contact', { name, email, publicKey });
}

/**
 * Remove trusted contact
 */
export async function removeTrustedContact(id: string): Promise<void> {
  return invoke('remove_trusted_contact', { id });
}

/**
 * Get trusted contacts
 */
export async function getTrustedContacts(): Promise<TrustedContact[]> {
  return invoke('get_trusted_contacts');
}

/**
 * Share password with contacts
 */
export async function sharePassword(title: string, url: string, username: string, encryptedPassword: string, contactIds: string[]): Promise<string> {
  return invoke('share_password', { title, url, username, encryptedPassword, contactIds });
}

/**
 * Revoke shared password
 */
export async function revokeSharedPassword(id: string): Promise<void> {
  return invoke('revoke_shared_password', { id });
}

/**
 * Get shared passwords
 */
export async function getSharedPasswords(): Promise<SharedPassword[]> {
  return invoke('get_shared_passwords');
}

/**
 * Set require approval
 */
export async function setPasswordSharingRequireApproval(require: boolean): Promise<void> {
  return invoke('set_password_sharing_require_approval', { require });
}

/**
 * Set auto expire days
 */
export async function setPasswordSharingAutoExpireDays(days: number): Promise<void> {
  return invoke('set_password_sharing_auto_expire_days', { days });
}

/**
 * Get password sharing settings
 */
export async function getPasswordSharingSettings(): Promise<PasswordSharingSettings> {
  return invoke('get_password_sharing_settings');
}
