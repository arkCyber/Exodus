import { describe, it, expect, beforeEach } from 'vitest';
import { useHistory } from './useHistory';

describe('useHistory', () => {
  beforeEach(() => {
    localStorage.clear();
    useHistory().clearHistory();
  });

  it('adds entry to history', () => {
    const { addToHistory, history } = useHistory();
    
    addToHistory('https://example.com', 'Example Page');
    
    expect(history.value.length).toBe(1);
    expect(history.value[0].url).toBe('https://example.com');
    expect(history.value[0].title).toBe('Example Page');
  });

  it('does not add data URLs to history', () => {
    const { addToHistory, history } = useHistory();
    
    addToHistory('data:text/plain,hello', 'Data URL');
    
    expect(history.value.length).toBe(0);
  });

  it('removes duplicate entries', () => {
    const { addToHistory, history } = useHistory();
    
    addToHistory('https://example.com', 'Example Page');
    addToHistory('https://example.com', 'Example Page Updated');
    
    expect(history.value.length).toBe(1);
    expect(history.value[0].title).toBe('Example Page Updated');
  });

  it('limits history size', () => {
    const { addToHistory, history } = useHistory();
    
    // Add more than MAX_HISTORY entries
    for (let i = 0; i < 1100; i++) {
      addToHistory(`https://example.com/${i}`, `Page ${i}`);
    }
    
    expect(history.value.length).toBe(1000);
  });

  it('searches history by title and URL', () => {
    const { addToHistory, searchHistory } = useHistory();
    
    addToHistory('https://example.com', 'Example Page');
    addToHistory('https://test.com', 'Test Page');
    addToHistory('https://demo.com', 'Demo Page');
    
    const results = searchHistory('test');
    
    expect(results.length).toBe(1);
    expect(results[0].url).toBe('https://test.com');
  });

  it('returns recent history', () => {
    const { addToHistory, getRecentHistory } = useHistory();
    
    addToHistory('https://example.com', 'Example Page');
    addToHistory('https://test.com', 'Test Page');
    addToHistory('https://demo.com', 'Demo Page');
    
    const recent = getRecentHistory(2);
    
    expect(recent.length).toBe(2);
    expect(recent[0].url).toBe('https://demo.com');
  });

  it('clears history', () => {
    const { addToHistory, clearHistory, history } = useHistory();
    
    addToHistory('https://example.com', 'Example Page');
    clearHistory();
    
    expect(history.value.length).toBe(0);
  });
});
