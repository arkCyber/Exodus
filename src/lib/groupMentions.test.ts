/**
 * Exodus Browser — group @mention helpers (unit tests).
 */
import { describe, expect, it } from 'vitest';
import type { Contact } from './contactDirectory';
import {
  extractMentionNodeIds,
  formatMentionToken,
  insertMentionToken,
  mentionPlainText,
  splitMentionContent,
  mergeMentionCandidates,
} from './groupMentions';

const contacts: Contact[] = [
  {
    contact_id: 'c1',
    name: 'Alice',
    contact_type: 'human',
    agent_ids: [],
    node_id: 'peer-node-alice-001',
    groups: [],
    tags: [],
    notes: '',
    is_favorite: false,
    is_blocked: false,
    created_at: 0,
    last_contacted: 0,
    contact_count: 0,
  },
];

describe('groupMentions', () => {
  it('formats and parses mention tokens', () => {
    const token = formatMentionToken({ nodeId: 'peer-node-alice-001', displayName: 'Alice' });
    expect(token).toContain('@[Alice](node:peer-node-alice-001)');
    const ids = extractMentionNodeIds(`Hi ${token}`, contacts);
    expect(ids).toContain('peer-node-alice-001');
  });

  it('insertMentionToken replaces trailing @query', () => {
    const next = insertMentionToken('hello @Ali', {
      nodeId: 'peer-node-alice-001',
      displayName: 'Alice',
    });
    expect(next).toContain('@[Alice](node:peer-node-alice-001)');
    expect(next).not.toContain('@Ali');
  });

  it('mentionPlainText strips tokens', () => {
    expect(mentionPlainText('@[Bob](node:x)')).toBe('@Bob');
  });

  it('mergeMentionCandidates includes group extras', () => {
    const merged = mergeMentionCandidates(contacts, [
      { nodeId: 'peer-node-bob-999', displayName: 'Bob' },
    ]);
    expect(merged.some((t) => t.nodeId === 'peer-node-alice-001')).toBe(true);
    expect(merged.some((t) => t.nodeId === 'peer-node-bob-999')).toBe(true);
  });

  it('splitMentionContent yields clickable segments', () => {
    const token = formatMentionToken({ nodeId: 'peer-node-alice-001', displayName: 'Alice' });
    const segs = splitMentionContent(`Hi ${token}!`);
    expect(segs.some((s) => s.kind === 'mention' && s.displayName === 'Alice')).toBe(true);
  });
});
