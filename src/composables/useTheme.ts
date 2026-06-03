import { ref, watch } from 'vue';

export type Theme = 'light' | 'dark' | 'auto';

const THEME_STORAGE_KEY = 'browser-theme';

const theme = ref<Theme>('auto');
const isDark = ref(false);

export function useTheme() {
  function applyTheme(themeValue: Theme) {
    if (themeValue === 'auto') {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      isDark.value = prefersDark;
    } else {
      isDark.value = themeValue === 'dark';
    }

    // Apply to document (html.dark + legacy light-theme + data-theme for settings CSS)
    document.documentElement.classList.toggle('dark', isDark.value);
    document.documentElement.classList.toggle('light-theme', !isDark.value);
    document.documentElement.setAttribute('data-theme', isDark.value ? 'dark' : 'light');
  }

  function setTheme(newTheme: Theme) {
    theme.value = newTheme;
    applyTheme(newTheme);
    
    // Persist to localStorage
    try {
      localStorage.setItem(THEME_STORAGE_KEY, newTheme);
    } catch (e) {
      console.error('Failed to save theme:', e);
    }
  }

  function loadTheme() {
    try {
      const saved = localStorage.getItem(THEME_STORAGE_KEY) as Theme | null;
      if (saved && (saved === 'light' || saved === 'dark' || saved === 'auto')) {
        theme.value = saved;
      } else if (saved !== null) {
        theme.value = 'auto';
      }
    } catch (e) {
      console.error('Failed to load theme:', e);
    }
    
    applyTheme(theme.value);
  }

  function toggleTheme() {
    if (theme.value === 'light') {
      setTheme('dark');
    } else if (theme.value === 'dark') {
      setTheme('auto');
    } else {
      setTheme('light');
    }
  }

  // Listen for system theme changes
  function setupSystemThemeListener() {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    
    const handleChange = (e: MediaQueryListEvent) => {
      if (theme.value === 'auto') {
        isDark.value = e.matches;
        document.documentElement.classList.toggle('dark', isDark.value);
        document.documentElement.classList.toggle('light-theme', !isDark.value);
        document.documentElement.setAttribute('data-theme', isDark.value ? 'dark' : 'light');
      }
    };

    mediaQuery.addEventListener('change', handleChange);
    
    return () => {
      mediaQuery.removeEventListener('change', handleChange);
    };
  }

  // Watch for theme changes
  watch(theme, (newTheme) => {
    applyTheme(newTheme);
  });

  return {
    theme,
    isDark,
    setTheme,
    loadTheme,
    toggleTheme,
    setupSystemThemeListener,
  };
}
