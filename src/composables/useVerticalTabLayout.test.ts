/**
 * Exodus Browser — useVerticalTabLayout composable tests.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useVerticalTabLayout } from './useVerticalTabLayout';

vi.mock('$lib/verticalTabs', () => ({
  isVerticalTabsRight: vi.fn((settings) => settings.position === 'right'),
  loadVerticalTabSettings: vi.fn(),
  readVerticalTabsCached: vi.fn(() => true),
  verticalTabStripWidth: vi.fn((settings) => settings.width ?? 220),
}));

describe('useVerticalTabLayout', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('initializes with null settings', () => {
    const { settings, verticalTabsOn, verticalTabWidth, verticalTabsRight } = useVerticalTabLayout();

    expect(settings.value).toBe(null);
    expect(verticalTabsOn.value).toBe(true);
    expect(verticalTabWidth.value).toBe(220);
    expect(verticalTabsRight.value).toBe(false);
  });

  it('computes vertical tabs on from settings', async () => {
    const { loadVerticalTabSettings } = await import('$lib/verticalTabs');
    const mockSettings = { enabled: true, position: 'left', width: 250 } as any;
    vi.mocked(loadVerticalTabSettings).mockResolvedValue(mockSettings);

    const { verticalTabsOn, loadVerticalLayout } = useVerticalTabLayout();

    await loadVerticalLayout();

    expect(verticalTabsOn.value).toBe(true);
  });

  it('computes vertical tab width from settings', async () => {
    const { loadVerticalTabSettings, verticalTabStripWidth } = await import('$lib/verticalTabs');
    const mockSettings = { enabled: true, position: 'left', width: 300 } as any;
    vi.mocked(loadVerticalTabSettings).mockResolvedValue(mockSettings);
    vi.mocked(verticalTabStripWidth).mockReturnValue(300);

    const { verticalTabWidth, loadVerticalLayout } = useVerticalTabLayout();

    await loadVerticalLayout();

    expect(verticalTabWidth.value).toBe(300);
  });

  it('computes vertical tabs right from settings', async () => {
    const { loadVerticalTabSettings, isVerticalTabsRight } = await import('$lib/verticalTabs');
    const mockSettings = { enabled: true, position: 'right', width: 220 } as any;
    vi.mocked(loadVerticalTabSettings).mockResolvedValue(mockSettings);
    vi.mocked(isVerticalTabsRight).mockReturnValue(true);

    const { verticalTabsRight, loadVerticalLayout } = useVerticalTabLayout();

    await loadVerticalLayout();

    expect(verticalTabsRight.value).toBe(true);
  });

  it('loads vertical layout settings', async () => {
    const { loadVerticalTabSettings } = await import('$lib/verticalTabs');
    const mockSettings = { enabled: true, position: 'left', width: 220 } as any;
    vi.mocked(loadVerticalTabSettings).mockResolvedValue(mockSettings);

    const { settings, loadVerticalLayout } = useVerticalTabLayout();

    await loadVerticalLayout();

    expect(settings.value).toEqual(mockSettings);
  });

  it('handles load errors gracefully', async () => {
    const { loadVerticalTabSettings } = await import('$lib/verticalTabs');
    vi.mocked(loadVerticalTabSettings).mockRejectedValue(new Error('Load failed'));

    const { settings, loadVerticalLayout } = useVerticalTabLayout();

    await loadVerticalLayout();

    expect(settings.value).toBe(null);
  });

  it('applies vertical layout settings', () => {
    const { settings, applyVerticalLayout } = useVerticalTabLayout();
    const newSettings = { enabled: false, position: 'left', width: 200 } as any;

    applyVerticalLayout(newSettings);

    expect(settings.value).toEqual(newSettings);
  });

  it('uses cached value when settings is null', async () => {
    const { readVerticalTabsCached } = await import('$lib/verticalTabs');
    vi.mocked(readVerticalTabsCached).mockReturnValue(false);

    const { verticalTabsOn } = useVerticalTabLayout();

    expect(verticalTabsOn.value).toBe(false);
  });
});
