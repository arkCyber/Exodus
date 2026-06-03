/**
 * Exodus Browser — reactive Firefox-style sidebar preferences.
 */
import { shellLog } from '@/lib/diagnosticLog';
import { ref, computed } from 'vue';
import {
  applySidebarPreferencesPatch,
  defaultSidebarPanel,
  loadSidebarPreferences,
  type SidebarPreferences,
  type SidebarToolId,
} from '@/lib/sidebarPreferences';
import { sidebarIconItemsFromPrefs } from '@/lib/sidebarIcons';
import { loadVerticalTabSettings, saveVerticalTabSettings } from '@/lib/verticalTabs';

/**
 * Sidebar layout preferences (position, tools, vertical tabs in sidebar).
 */
export function useSidebarPreferences() {
  const prefs = ref<SidebarPreferences>(loadSidebarPreferences());

  const iconItems = computed(() => sidebarIconItemsFromPrefs(prefs.value));
  const sidebarOnLeft = computed(() => prefs.value.position === 'left');
  const verticalTabsInSidebar = computed(() => prefs.value.verticalTabsInSidebar);
  const hideHorizontalTabBar = computed(() => prefs.value.verticalTabsInSidebar);

  function updatePrefs(patch: Partial<SidebarPreferences>): void {
    prefs.value = applySidebarPreferencesPatch(prefs.value, patch);
  }

  function toggleTool(tool: SidebarToolId): void {
    const set = new Set(prefs.value.enabledTools);
    if (set.has(tool)) {
      if (set.size <= 1) return;
      set.delete(tool);
    } else {
      set.add(tool);
    }
    const patch: Partial<SidebarPreferences> = { enabledTools: [...set] };
    if (tool === 'tabs' && !set.has('tabs') && prefs.value.verticalTabsInSidebar) {
      patch.verticalTabsInSidebar = false;
    }
    updatePrefs(patch);
    if (patch.verticalTabsInSidebar === false) {
      void setVerticalTabsInSidebar(false);
    }
  }

  function setPosition(position: 'left' | 'right'): void {
    updatePrefs({ position });
    void syncVerticalTabPosition(position);
  }

  async function setVerticalTabsInSidebar(enabled: boolean): Promise<void> {
    updatePrefs({ verticalTabsInSidebar: enabled });
    if (enabled && !prefs.value.enabledTools.includes('tabs')) {
      updatePrefs({ enabledTools: ['tabs', ...prefs.value.enabledTools] });
    }
    try {
      const vt = await loadVerticalTabSettings();
      const position = prefs.value.position === 'left' ? 'Left' : 'Right';
      await saveVerticalTabSettings({
        ...vt,
        enabled,
        position,
      });
    } catch (error) {
      shellLog.error('setVerticalTabsInSidebar saveVerticalTabSettings failed', error);
    }
  }

  async function syncVerticalTabPosition(sidebarPosition: 'left' | 'right'): Promise<void> {
    try {
      const vt = await loadVerticalTabSettings();
      await saveVerticalTabSettings({
        ...vt,
        position: sidebarPosition === 'left' ? 'Left' : 'Right',
      });
    } catch (error) {
      shellLog.error('syncVerticalTabPosition failed', error);
    }
  }

  async function loadPrefs(): Promise<void> {
    prefs.value = loadSidebarPreferences();
    try {
      const vt = await loadVerticalTabSettings();
      if (vt.enabled !== prefs.value.verticalTabsInSidebar) {
        updatePrefs({ verticalTabsInSidebar: vt.enabled });
      }
    } catch (error) {
      shellLog.error('loadPrefs vertical tabs failed', error);
    }
  }

  function resolvePanel(requested: import('$lib/browserTypes').SidebarPanel): import('$lib/browserTypes').SidebarPanel {
    shellLog.info('resolvePanel called with', requested);
    shellLog.info('enabledTools', prefs.value.enabledTools);
    if (requested === 'customize') return 'customize';
    if (prefs.value.enabledTools.includes(requested as SidebarToolId)) {
      shellLog.info('Panel is enabled, returning', requested);
      return requested;
    }
    const fallback = defaultSidebarPanel(prefs.value);
    shellLog.info('Panel not enabled, falling back to', fallback);
    return fallback;
  }

  return {
    prefs,
    iconItems,
    sidebarOnLeft,
    verticalTabsInSidebar,
    hideHorizontalTabBar,
    updatePrefs,
    toggleTool,
    setPosition,
    setVerticalTabsInSidebar,
    loadPrefs,
    resolvePanel,
    defaultPanel: () => defaultSidebarPanel(prefs.value),
  };
}
