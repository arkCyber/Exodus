/**
 * Exodus Browser — useTheme composable tests.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useTheme } from './useTheme';

describe('useTheme', () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.classList.remove('dark', 'light-theme');
    document.documentElement.removeAttribute('data-theme');
    const { setTheme } = useTheme();
    setTheme('auto');
  });

  it('sets theme to light', () => {
    const { setTheme, isDark } = useTheme();
    setTheme('light');
    
    expect(isDark.value).toBe(false);
    expect(document.documentElement.classList.contains('dark')).toBe(false);
    expect(document.documentElement.classList.contains('light-theme')).toBe(true);
    expect(document.documentElement.getAttribute('data-theme')).toBe('light');
  });

  it('sets theme to dark', () => {
    const { setTheme, isDark } = useTheme();
    setTheme('dark');
    
    expect(isDark.value).toBe(true);
    expect(document.documentElement.classList.contains('dark')).toBe(true);
    expect(document.documentElement.classList.contains('light-theme')).toBe(false);
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark');
  });

  it('sets theme to auto and respects system preference', () => {
    vi.stubGlobal('matchMedia', (query: string) => ({
      matches: query === '(prefers-color-scheme: dark)',
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    }));

    const { setTheme, isDark } = useTheme();
    setTheme('auto');
    
    expect(isDark.value).toBe(true);
    expect(document.documentElement.classList.contains('dark')).toBe(true);
    
    vi.unstubAllGlobals();
  });

  it('toggles theme from light to dark', () => {
    const { setTheme, toggleTheme, theme } = useTheme();
    setTheme('light');
    
    toggleTheme();
    
    expect(theme.value).toBe('dark');
  });

  it('toggles theme from dark to auto', () => {
    const { setTheme, toggleTheme, theme } = useTheme();
    setTheme('dark');
    
    toggleTheme();
    
    expect(theme.value).toBe('auto');
  });

  it('toggles theme from auto to light', () => {
    const { setTheme, toggleTheme, theme } = useTheme();
    setTheme('auto');
    
    toggleTheme();
    
    expect(theme.value).toBe('light');
  });

  it('saves theme to localStorage', () => {
    const { setTheme } = useTheme();
    setTheme('dark');
    
    expect(localStorage.getItem('browser-theme')).toBe('dark');
  });

  it('loads theme from localStorage', () => {
    localStorage.setItem('browser-theme', 'dark');
    
    const { loadTheme, theme, isDark } = useTheme();
    loadTheme();
    
    expect(theme.value).toBe('dark');
    expect(isDark.value).toBe(true);
  });

  it('handles invalid saved theme', () => {
    localStorage.setItem('browser-theme', 'invalid' as any);
    
    const { loadTheme, theme } = useTheme();
    loadTheme();
    
    expect(theme.value).toBe('auto');
  });

  it('handles localStorage errors gracefully', () => {
    const getItemSpy = vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
      throw new Error('Storage error');
    });
    
    const { loadTheme, theme } = useTheme();
    loadTheme();
    
    expect(theme.value).toBe('auto');
    getItemSpy.mockRestore();
  });

  it('sets up system theme listener', () => {
    const addEventListener = vi.fn();
    const removeEventListener = vi.fn();
    vi.stubGlobal('matchMedia', () => ({
      matches: false,
      media: '',
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener,
      removeEventListener,
      dispatchEvent: vi.fn(),
    }));

    const { setupSystemThemeListener } = useTheme();
    const cleanup = setupSystemThemeListener();

    expect(addEventListener).toHaveBeenCalledWith('change', expect.any(Function));

    cleanup();
    vi.unstubAllGlobals();
  });

  it('cleans up system theme listener', () => {
    const addEventListener = vi.fn();
    const removeEventListener = vi.fn();
    vi.stubGlobal('matchMedia', () => ({
      matches: false,
      media: '',
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener,
      removeEventListener,
      dispatchEvent: vi.fn(),
    }));

    const { setupSystemThemeListener } = useTheme();
    const cleanup = setupSystemThemeListener();
    cleanup();

    expect(removeEventListener).toHaveBeenCalledWith('change', expect.any(Function));
    vi.unstubAllGlobals();
  });

  it('reacts to system theme changes when in auto mode', () => {
    vi.stubGlobal('matchMedia', (query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: (event: string, callback: (e: MediaQueryListEvent) => void) => {
        if (event === 'change') {
          setTimeout(() => callback({ matches: true, media: query } as MediaQueryListEvent), 0);
        }
      },
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    }));

    const { setTheme, isDark, setupSystemThemeListener } = useTheme();
    setTheme('auto');
    
    expect(isDark.value).toBe(false);
    
    const cleanup = setupSystemThemeListener();
    
    // Wait for the change event to be processed
    return new Promise(resolve => {
      setTimeout(() => {
        expect(isDark.value).toBe(true);
        cleanup();
        vi.unstubAllGlobals();
        resolve(undefined);
      }, 10);
    });
  });
});
