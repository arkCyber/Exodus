/**
 * Exodus Browser — useClosedTabs unit tests.
 */
import { describe, it, expect } from 'vitest';
import { useClosedTabs } from './useClosedTabs';

describe('useClosedTabs', () => {
  it('records and pops most recent tab', () => {
    const stack = useClosedTabs(2);
    stack.recordClosedTab({ title: 'A', url: 'https://a.test' });
    stack.recordClosedTab({ title: 'B', url: 'https://b.test' });
    expect(stack.closedTabsCount.value).toBe(2);
    expect(stack.popClosedTab()?.url).toBe('https://b.test');
    expect(stack.popClosedTab()?.url).toBe('https://a.test');
    expect(stack.popClosedTab()).toBeUndefined();
  });

  it('caps stack size', () => {
    const stack = useClosedTabs(2);
    stack.recordClosedTab({ title: '1', url: 'https://1.test' });
    stack.recordClosedTab({ title: '2', url: 'https://2.test' });
    stack.recordClosedTab({ title: '3', url: 'https://3.test' });
    expect(stack.closedTabsCount.value).toBe(2);
    expect(stack.popClosedTab()?.url).toBe('https://3.test');
  });
});
