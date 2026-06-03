/**
 * Exodus Browser — passwordNeverSave unit tests.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest';

const store = new Map<string, string>();

vi.stubGlobal('localStorage', {
  getItem: (key: string) => store.get(key) ?? null,
  setItem: (key: string, value: string) => {
    store.set(key, value);
  },
  removeItem: (key: string) => {
    store.delete(key);
  },
  clear: () => store.clear(),
});

import {
  addNeverSavePasswordHost,
  getNeverSaveHosts,
  isNeverSavePasswordUrl,
} from './passwordNeverSave';

describe('passwordNeverSave', () => {
  beforeEach(() => {
    store.clear();
  });

  it('stores and checks never-save host', () => {
    expect(isNeverSavePasswordUrl('https://login.example.com/path')).toBe(false);
    addNeverSavePasswordHost('https://Login.Example.com/');
    expect(getNeverSaveHosts()).toEqual(['login.example.com']);
    expect(isNeverSavePasswordUrl('https://login.example.com/')).toBe(true);
  });

  it('returns false for invalid url', () => {
    expect(isNeverSavePasswordUrl('not-a-url')).toBe(false);
  });
});
