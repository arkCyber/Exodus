/**
 * Exodus Browser — WebRTC config tests.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  DEFAULT_ICE_SERVERS,
  DEFAULT_PC_CONFIG,
  loadCustomTurnUrls,
  saveCustomTurnUrls,
  buildIceServers,
  getRtcConfiguration,
} from './rtcConfig';

describe('rtcConfig', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
  });

  it('has default ICE servers', () => {
    expect(DEFAULT_ICE_SERVERS).toEqual([
      { urls: 'stun:stun.l.google.com:19302' },
      { urls: 'stun:stun1.l.google.com:19302' },
    ]);
  });

  it('has default PC config', () => {
    expect(DEFAULT_PC_CONFIG).toEqual({
      iceServers: DEFAULT_ICE_SERVERS,
    });
  });

  it('loads custom TURN URLs from localStorage', () => {
    localStorage.setItem('exodus-turn-ice-servers', 'turn:server1.com\nturn:server2.com');

    const urls = loadCustomTurnUrls();

    expect(urls).toEqual(['turn:server1.com', 'turn:server2.com']);
  });

  it('returns empty array when localStorage is undefined', () => {
    const originalLocalStorage = global.localStorage;
    // @ts-ignore
    delete global.localStorage;

    const urls = loadCustomTurnUrls();

    expect(urls).toEqual([]);

    global.localStorage = originalLocalStorage;
  });

  it('returns empty array when no TURN URLs saved', () => {
    const urls = loadCustomTurnUrls();

    expect(urls).toEqual([]);
  });

  it('saves custom TURN URLs to localStorage', () => {
    saveCustomTurnUrls(['turn:server1.com', 'turn:server2.com']);

    const saved = localStorage.getItem('exodus-turn-ice-servers');
    expect(saved).toBe('turn:server1.com\nturn:server2.com');
  });

  it('does nothing when localStorage is undefined on save', () => {
    const originalLocalStorage = global.localStorage;
    // @ts-ignore
    delete global.localStorage;

    saveCustomTurnUrls(['turn:server1.com']);

    global.localStorage = originalLocalStorage;
  });

  it('filters empty URLs when saving', () => {
    saveCustomTurnUrls(['turn:server1.com', '', 'turn:server2.com', '']);

    const saved = localStorage.getItem('exodus-turn-ice-servers');
    expect(saved).toBe('turn:server1.com\nturn:server2.com');
  });

  it('builds ICE servers with defaults', () => {
    const servers = buildIceServers([]);

    expect(servers).toEqual(DEFAULT_ICE_SERVERS);
  });

  it('builds ICE servers with custom TURN URLs', () => {
    const servers = buildIceServers(['turn:custom.com']);

    expect(servers).toEqual([
      { urls: 'stun:stun.l.google.com:19302' },
      { urls: 'stun:stun1.l.google.com:19302' },
      { urls: 'turn:custom.com' },
    ]);
  });

  it('builds ICE servers from localStorage by default', () => {
    localStorage.setItem('exodus-turn-ice-servers', 'turn:custom.com');

    const servers = buildIceServers();

    expect(servers).toContainEqual({ urls: 'turn:custom.com' });
  });

  it('gets RTC configuration', () => {
    const config = getRtcConfiguration();

    expect(config).toEqual({
      iceServers: expect.arrayContaining(DEFAULT_ICE_SERVERS),
    });
  });
});
