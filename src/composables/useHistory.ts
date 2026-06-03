import { shallowRef } from 'vue';

interface HistoryEntry {
  id: string;
  url: string;
  title: string;
  timestamp: number;
  favicon?: string;
}

const MAX_HISTORY = 1000;
const history = shallowRef<HistoryEntry[]>([]);

export function useHistory() {
  function addToHistory(url: string, title: string, favicon?: string) {
    // Validate URL
    if (!url || url.startsWith('data:') || url.startsWith('javascript:')) return;
    
    // Validate title
    const safeTitle = title?.trim() || url;
    
    // Normalize URL
    let normalizedUrl = url.trim();
    if (!normalizedUrl.startsWith('http://') && !normalizedUrl.startsWith('https://')) {
      normalizedUrl = `https://${normalizedUrl}`;
    }
    
    // Remove duplicates
    const existingIndex = history.value.findIndex(h => h.url === normalizedUrl);
    if (existingIndex > -1) {
      history.value.splice(existingIndex, 1);
    }
    
    // Add new entry
    const entry: HistoryEntry = {
      id: `history-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      url: normalizedUrl,
      title: safeTitle,
      timestamp: Date.now(),
      favicon,
    };
    
    history.value.unshift(entry);
    
    // Limit history size
    if (history.value.length > MAX_HISTORY) {
      history.value = history.value.slice(0, MAX_HISTORY);
    }
    
    // Persist to localStorage with quota handling
    try {
      localStorage.setItem('browser-history', JSON.stringify(history.value));
    } catch (e) {
      if (e instanceof DOMException && e.name === 'QuotaExceededError') {
        console.warn('LocalStorage quota exceeded, clearing old history');
        history.value = history.value.slice(0, MAX_HISTORY / 2);
        try {
          localStorage.setItem('browser-history', JSON.stringify(history.value));
        } catch (retryError) {
          console.error('Failed to save history after cleanup:', retryError);
        }
      } else {
        console.error('Failed to save history:', e);
      }
    }
  }
  
  function loadHistory() {
    try {
      const saved = localStorage.getItem('browser-history');
      if (saved) {
        const parsed = JSON.parse(saved);
        // Validate parsed data is an array
        if (Array.isArray(parsed)) {
          history.value = parsed.filter((h: any) => 
            h && typeof h === 'object' && h.id && h.url
          );
        }
      }
    } catch (e) {
      console.error('Failed to load history:', e);
      // Reset to empty on error
      history.value = [];
    }
  }
  
  function clearHistory() {
    history.value = [];
    try {
      localStorage.removeItem('browser-history');
    } catch (e) {
      console.error('Failed to clear history:', e);
    }
  }
  
  function searchHistory(query: string): HistoryEntry[] {
    if (!query) return history.value;
    const lowerQuery = query.toLowerCase();
    return history.value.filter(h => 
      h.title.toLowerCase().includes(lowerQuery) || 
      h.url.toLowerCase().includes(lowerQuery)
    );
  }
  
  function getRecentHistory(limit: number = 10): HistoryEntry[] {
    return history.value.slice(0, limit);
  }
  
  return {
    history,
    addToHistory,
    loadHistory,
    clearHistory,
    searchHistory,
    getRecentHistory,
  };
}
