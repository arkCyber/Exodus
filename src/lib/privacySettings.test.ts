/**
 * Tests for privacy settings helpers.
 */
import { describe, expect, it } from 'vitest';
import {
  applyHttpsOnly,
  iframeSandboxAttr,
  parsePrivacyTuple,
  shouldPersistSession,
} from './privacySettings';

describe('parsePrivacyTuple', () => {
  it('maps backend tuple to named settings', () => {
    expect(parsePrivacyTuple([true, false, true, false])).toEqual({
      httpsOnly: true,
      privateMode: false,
      blockPopups: true,
      sessionRestore: false,
    });
  });
});

describe('applyHttpsOnly', () => {
  it('upgrades http to https when enabled', () => {
    expect(applyHttpsOnly('http://example.com/path', true)).toBe('https://example.com/path');
  });

  it('leaves https URLs unchanged', () => {
    expect(applyHttpsOnly('https://example.com', true)).toBe('https://example.com');
  });

  it('does nothing when HTTPS-only is off', () => {
    expect(applyHttpsOnly('http://example.com', false)).toBe('http://example.com');
  });
});

describe('shouldPersistSession', () => {
  it('is false in private mode even when session restore is on', () => {
    expect(shouldPersistSession(true, true)).toBe(false);
  });

  it('is true only when restore is on and not private', () => {
    expect(shouldPersistSession(true, false)).toBe(true);
    expect(shouldPersistSession(false, false)).toBe(false);
  });
});

describe('iframeSandboxAttr', () => {
  it('removes allow-popups when blocking', () => {
    expect(iframeSandboxAttr(true)).not.toContain('allow-popups');
    expect(iframeSandboxAttr(true)).toContain('allow-scripts');
  });

  it('allows popups when blocking is off', () => {
    expect(iframeSandboxAttr(false)).toContain('allow-popups');
  });
});
