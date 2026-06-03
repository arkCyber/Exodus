/**
 * Integration tests for sidebar open/resolve helpers (Firefox Customize).
 */
import { describe, it, expect, beforeEach } from 'vitest';
import {
  applySidebarPreferencesPatch,
  loadSidebarPreferences,
  defaultSidebarPanel,
  isToolEnabled,
} from '@/lib/sidebarPreferences';
import type { SidebarPanel } from '@/lib/browserTypes';

function resolvePanel(
  prefs: ReturnType<typeof loadSidebarPreferences>,
  requested: SidebarPanel,
): SidebarPanel {
  if (requested === 'customize') return 'customize';
  if (isToolEnabled(prefs, requested as import('@/lib/sidebarPreferences').SidebarToolId)) {
    return requested;
  }
  return defaultSidebarPanel(prefs);
}

describe('sidebar panel resolution', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('falls back when pocket is disabled', () => {
    const prefs = applySidebarPreferencesPatch(loadSidebarPreferences(), {
      enabledTools: ['ai', 'memory'],
    });
    expect(resolvePanel(prefs, 'pocket')).toBe('ai');
  });

  it('allows customize regardless of tools', () => {
    const prefs = applySidebarPreferencesPatch(loadSidebarPreferences(), {
      enabledTools: ['ai'],
    });
    expect(resolvePanel(prefs, 'customize')).toBe('customize');
  });

  it('defaults to tabs when vertical tabs enabled', () => {
    const prefs = applySidebarPreferencesPatch(loadSidebarPreferences(), {
      verticalTabsInSidebar: true,
      enabledTools: ['tabs', 'ai'],
    });
    expect(defaultSidebarPanel(prefs)).toBe('tabs');
  });
});
