/**
 * Exodus Browser — Contact Directory API (P2P address book).
 */
import { invoke } from '@tauri-apps/api/core';

export type Contact = {
  contact_id: string;
  name: string;
  contact_type: string;
  agent_deployment_type?: string | null;
  agent_ids: string[];
  node_id: string;
  groups: string[];
  tags: string[];
  notes: string;
  is_favorite: boolean;
  is_blocked: boolean;
  created_at: number;
  last_contacted: number;
  contact_count: number;
  public_account_id?: string | null;
};

export type ContactDirectoryHubInfo = {
  storageDir: string;
  nodeId: string;
  inProcess: boolean;
};

export async function contactDirectoryServiceStart(): Promise<ContactDirectoryHubInfo> {
  return invoke<ContactDirectoryHubInfo>('contact_directory_service_start');
}

export async function contactDirectoryHubInfo(): Promise<ContactDirectoryHubInfo> {
  return invoke<ContactDirectoryHubInfo>('contact_directory_hub_info');
}

/** This device's 12-digit Exodus ID (stable for the P2P node). */
export async function contactGetLocalDigit(): Promise<string> {
  return invoke<string>('contact_get_local_digit');
}

export type ContactExportBundle = {
  version: number;
  exportedAt: number;
  contacts: Contact[];
  groups: Array<{
    group_id: string;
    name: string;
    description: string;
    color: string;
    created_at: number;
  }>;
};

/** Export all contacts + groups as JSON (pretty-printed). */
export async function contactExportJson(): Promise<string> {
  return invoke<string>('contact_export_json');
}

/** Import contacts from JSON; `merge` updates by node id instead of replacing all. */
export async function contactImportJson(json: string, merge = true): Promise<number> {
  return invoke<number>('contact_import_json', { json, merge });
}

/** Download export JSON as a file in the browser. */
export function downloadContactExport(json: string, filename = 'exodus-contacts.json'): void {
  const blob = new Blob([json], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

export async function contactList(): Promise<Contact[]> {
  return invoke<Contact[]>('contact_list');
}

export async function contactAdd(contact: Contact): Promise<void> {
  await invoke('contact_add', { contact });
}

export async function contactRemove(contactId: string): Promise<void> {
  await invoke('contact_remove', { contactId });
}

export async function contactSearch(query: string): Promise<Contact[]> {
  return invoke<Contact[]>('contact_search', { query });
}

export async function contactResolveDigitToNode(digitId: string): Promise<string> {
  return invoke<string>('contact_resolve_digit_to_node', { digitId });
}

export async function contactAddFriendByDigit(
  digitId: string,
  name: string,
  userId: string
): Promise<Contact> {
  return invoke<Contact>('contact_add_friend_by_digit', { digitId, name, userId });
}

export async function contactGetDigitForNode(nodeId: string): Promise<string> {
  return invoke<string>('contact_get_digit_for_node', { nodeId });
}

export async function contactGetByNode(nodeId: string): Promise<Contact[]> {
  return invoke<Contact[]>('contact_get_by_node', { nodeId });
}

export async function contactUpdate(contact: Contact): Promise<void> {
  await invoke('contact_update', { contact });
}

export async function contactRegisterDigitMapping(digitId: string, nodeId: string): Promise<void> {
  await invoke('contact_register_digit_mapping', { digitId, nodeId });
}

/** Bump `last_contacted` after a DM message (best-effort). */
export async function touchContactLastContacted(nodeId: string): Promise<void> {
  try {
    const rows = await contactGetByNode(nodeId);
    const c = rows[0];
    if (!c) return;
    c.last_contacted = Math.floor(Date.now() / 1000);
    c.contact_count += 1;
    await contactUpdate(c);
  } catch {
    /* directory may be offline */
  }
}

/** Build a new human contact for `contact_add`. */
export function buildHumanContact(params: {
  name: string;
  nodeId: string;
  notes?: string;
  groups?: string[];
}): Contact {
  const now = Math.floor(Date.now() / 1000);
  return {
    contact_id: crypto.randomUUID(),
    name: params.name,
    contact_type: 'human',
    agent_deployment_type: null,
    agent_ids: [],
    node_id: params.nodeId,
    groups: params.groups ?? ['friends'],
    tags: [],
    notes: params.notes ?? '',
    is_favorite: false,
    is_blocked: false,
    created_at: now,
    last_contacted: now,
    contact_count: 0,
    public_account_id: null,
  };
}
