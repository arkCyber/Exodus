/**
 * Exodus Browser — vertical tab strip layout settings.
 */
import { ref, computed } from 'vue';
import {
  isVerticalTabsRight,
  loadVerticalTabSettings,
  readVerticalTabsCached,
  verticalTabStripWidth,
  type VerticalTabSettings,
} from '@/lib/verticalTabs';

/**
 * Reactive vertical tabs layout (left/right strip vs horizontal bar).
 */
export function useVerticalTabLayout() {
  const settings = ref<VerticalTabSettings | null>(null);

  const verticalTabsOn = computed(() => {
    if (settings.value) return settings.value.enabled;
    const cached = readVerticalTabsCached();
    return cached ?? false;
  });

  const verticalTabWidth = computed(() =>
    settings.value ? verticalTabStripWidth(settings.value) : 220,
  );

  const verticalTabsRight = computed(() =>
    settings.value ? isVerticalTabsRight(settings.value) : false,
  );

  async function loadVerticalLayout(): Promise<void> {
    try {
      settings.value = await loadVerticalTabSettings();
    } catch (error) {
      console.error('loadVerticalTabSettings failed:', error);
    }
  }

  function applyVerticalLayout(next: VerticalTabSettings): void {
    settings.value = next;
  }

  return {
    settings,
    verticalTabsOn,
    verticalTabWidth,
    verticalTabsRight,
    loadVerticalLayout,
    applyVerticalLayout,
  };
}
