/**
 * Exodus Browser — exodusAllama extension helpers tests.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { getExodusAllamaShim, exodusAllamaAvailable } from './exodusAllama';

describe('exodusAllama', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('returns null when window is undefined', () => {
    const originalWindow = global.window;
    // @ts-ignore
    delete global.window;

    const shim = getExodusAllamaShim();

    expect(shim).toBe(null);

    global.window = originalWindow;
  });

  it('returns null when exodus is not defined', () => {
    // @ts-ignore
    window.exodus = undefined;

    const shim = getExodusAllamaShim();

    expect(shim).toBe(null);
  });

  it('returns null when allama is not defined', () => {
    // @ts-ignore
    window.exodus = {};

    const shim = getExodusAllamaShim();

    expect(shim).toBe(null);
  });

  it('returns shim when available', () => {
    const mockShim = {
      port: 11435,
      baseUrl: 'http://127.0.0.1:11435',
      health: vi.fn(),
      chat: vi.fn(),
      generate: vi.fn(),
      embed: vi.fn(),
      streamChat: vi.fn(),
    };
    // @ts-ignore
    window.exodus = { allama: mockShim };

    const shim = getExodusAllamaShim();

    expect(shim).toEqual(mockShim);
  });

  it('returns false for availability when shim is null', () => {
    // @ts-ignore
    window.exodus = undefined;

    const available = exodusAllamaAvailable();

    expect(available).toBe(false);
  });

  it('returns true for availability when shim exists', () => {
    const mockShim = {
      port: 11435,
      baseUrl: 'http://127.0.0.1:11435',
      health: vi.fn(),
      chat: vi.fn(),
      generate: vi.fn(),
      embed: vi.fn(),
      streamChat: vi.fn(),
    };
    // @ts-ignore
    window.exodus = { allama: mockShim };

    const available = exodusAllamaAvailable();

    expect(available).toBe(true);
  });
});
