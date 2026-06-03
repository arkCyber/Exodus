/**
 * Exodus Browser — extensionDisplayName unit tests.
 */
import { describe, expect, it } from 'vitest';
import { extensionDisplayName } from './extensionDisplayName';
import type { ExtensionInfo } from './types';

const catalog: ExtensionInfo[] = [
  {
    id: 'abc123',
    name: 'Catalog Name',
    version: '1.0.0',
    enabled: true,
    permissions: [],
    path: '/tmp/abc',
  },
];

describe('extensionDisplayName', () => {
  it('uses extensionName from event when present', () => {
    expect(
      extensionDisplayName({
        extensionId: 'abc123',
        extensionName: 'From Event',
      }),
    ).toBe('From Event');
  });

  it('falls back to installed catalog name', () => {
    expect(
      extensionDisplayName(
        { extensionId: 'abc123', extensionName: '' },
        catalog,
      ),
    ).toBe('Catalog Name');
  });

  it('falls back to extension id', () => {
    expect(
      extensionDisplayName({ extensionId: 'unknown-id', extensionName: '' }, catalog),
    ).toBe('unknown-id');
  });

  it('returns Extension when request is null', () => {
    expect(extensionDisplayName(null)).toBe('Extension');
  });
});
