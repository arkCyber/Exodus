/**
 * Exodus Browser — promptQueue unit tests.
 */
import { describe, expect, it } from 'vitest';
import { advancePromptQueue, enqueuePrompt } from './promptQueue';

describe('promptQueue', () => {
  it('enqueue shows first item immediately', () => {
    const next = enqueuePrompt<string | null>(null, [], 'a');
    expect(next.active).toBe('a');
    expect(next.queue).toEqual([]);
  });

  it('enqueue stacks when active exists', () => {
    const next = enqueuePrompt('a', [], 'b');
    expect(next.active).toBe('a');
    expect(next.queue).toEqual(['b']);
  });

  it('advance drains queue FIFO', () => {
    const q = ['b', 'c'];
    expect(advancePromptQueue(q)).toBe('b');
    expect(advancePromptQueue(q)).toBe('c');
    expect(advancePromptQueue(q)).toBeNull();
  });
});
