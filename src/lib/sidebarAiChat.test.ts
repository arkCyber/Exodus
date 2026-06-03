/**
 * Tests for sidebar AI chat message building.
 */
import { describe, expect, it } from 'vitest';
import { chatMessagesFromHistory } from './sidebarAiChat';

describe('sidebarAiChat', () => {
  it('prepends system message and keeps history order', () => {
    const msgs = chatMessagesFromHistory([
      { role: 'user', content: 'hello' },
      { role: 'assistant', content: 'hi' },
      { role: 'user', content: 'again' },
    ]);
    expect(msgs[0].role).toBe('system');
    expect(msgs[1]).toEqual({ role: 'user', content: 'hello' });
    expect(msgs[msgs.length - 1]).toEqual({ role: 'user', content: 'again' });
  });
});
