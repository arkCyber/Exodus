/**
 * Unit tests for site shield helpers.
 */

import { describe, expect, it } from 'vitest';
import { hostFromPageUrl } from './siteShields';

describe('hostFromPageUrl', () => {
  it('returns hostname from https URL', () => {
    expect(hostFromPageUrl('https://www.example.com/path')).toBe('www.example.com');
  });

  it('returns empty for invalid URL', () => {
    expect(hostFromPageUrl('not-a-url')).toBe('');
  });
});
