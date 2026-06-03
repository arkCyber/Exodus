/**
 * Exodus Browser — ImMessenger Vue 3 integration tests (replaces Svelte stub tests).
 */
import { describe, it, expect } from 'vitest';
import {
  CONTACT_DIRECTORY_CATEGORIES,
  primaryNavItems,
  IM_MUTED_CHATS_KEY,
} from '$lib/imMessengerWebchat';
import { IM_OPEN_CONTACT_EVENT, IM_START_CALL_EVENT, OPEN_WEBCHAT_EVENT } from '$lib/imChat';

describe('ImMessenger (Vue 3)', () => {
  it('uses WebChat desktop nav order distinct from sidebar mode', () => {
    const desktop = primaryNavItems(true).map((item) => item.id);
    const sidebar = primaryNavItems(false).map((item) => item.id);
    expect(desktop[1]).toBe('contacts');
    expect(sidebar[1]).toBe('collections');
  });

  it('defines macOS-style contact directory categories', () => {
    expect(CONTACT_DIRECTORY_CATEGORIES.map((item) => item.id)).toEqual([
      'new_friends',
      'group_chats',
      'official_accounts',
      'service_accounts',
      'wecom_contacts',
      'my_enterprises',
      'contacts',
    ]);
  });

  it('exports Vue IM event bus constants (not Svelte-only)', () => {
    expect(IM_OPEN_CONTACT_EVENT).toBe('exodus-open-im');
    expect(IM_START_CALL_EVENT).toBe('exodus-start-call');
    expect(OPEN_WEBCHAT_EVENT).toBe('exodus-open-webchat-ui');
  });

  it('persists mute map under stable localStorage key', () => {
    expect(IM_MUTED_CHATS_KEY).toBe('exodus-im-muted-chats');
  });
});
