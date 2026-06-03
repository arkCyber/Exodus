/**
 * Exodus Browser — presence TTL constants (unit tests).
 */
import { describe, expect, it } from 'vitest';
import { PRESENCE_TTL_MS, PRESENCE_TOPIC } from './presence';

describe('presence', () => {
  it('uses stable gossip topic', () => {
    expect(PRESENCE_TOPIC).toBe('exodus-presence');
  });

  it('TTL is at least one minute', () => {
    expect(PRESENCE_TTL_MS).toBeGreaterThanOrEqual(60_000);
  });
});
