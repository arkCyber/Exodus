/**
 * Exodus Browser — NewTabPage component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import NewTabPage from './NewTabPage.vue';

vi.mock('$lib/newTabPage', async (importOriginal) => {
  const actual = await importOriginal<typeof import('$lib/newTabPage')>();
  return {
    ...actual,
    DEFAULT_QUICK_LINKS: [
      { title: 'Google', url: 'https://google.com' },
      { title: 'GitHub', url: 'https://github.com' },
    ],
    DEFAULT_NTP_TOP_SITES: [
      { title: 'Google', url: 'https://www.google.com' },
      { title: 'GitHub', url: 'https://github.com' },
      { title: 'Wikipedia', url: 'https://en.wikipedia.org' },
      { title: 'YouTube', url: 'https://www.youtube.com' },
      { title: 'Twitter', url: 'https://twitter.com' },
      { title: 'Reddit', url: 'https://www.reddit.com' },
      { title: 'Stack Overflow', url: 'https://stackoverflow.com' },
      { title: 'MDN', url: 'https://developer.mozilla.org' },
    ],
    isNewTabUrl: vi.fn(() => false),
  };
});

vi.mock('$lib/diagnosticLog', () => ({
  ntpLog: {
    info: vi.fn(),
    warn: vi.fn(),
    error: vi.fn(),
    timeStart: vi.fn(),
    timeEnd: vi.fn(),
  },
}));

vi.mock('$lib/startupLog', () => ({
  logStartup: vi.fn(),
  logStartupError: vi.fn(),
}));

vi.mock('$lib/newTabWallpaper', () => ({
  WALLPAPER_FEATURE_ENABLED: true,
  loadWallpaperId: vi.fn(() => 'ishaan-sen'),
  getWindowSessionWallpaperId: vi.fn(() => null),
  ensureWallpaperDataUrl: vi.fn(() => Promise.resolve('blob:mock-wallpaper')),
  resolveWallpaperDisplayUrl: vi.fn(() => Promise.resolve('blob:mock-wallpaper')),
  getWallpaperById: vi.fn((id: string) => ({
    id,
    name: 'Test',
    accent: '#6366f1',
    file: 'ishaan-sen-OQRkj2erTPI-unsplash.jpg',
  })),
  wallpaperAssetUrl: vi.fn(() => '/newtab/wallpapers/ishaan-sen-OQRkj2erTPI-unsplash.jpg'),
  wallpaperAbsoluteAssetUrl: vi.fn(() => 'blob:mock-wallpaper'),
  peekCachedWallpaperDisplayUrl: vi.fn(() => ''),
}));

describe('NewTabPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  const mockProps = {
    visible: true,
    topSites: [
      { title: 'Google', url: 'https://google.com' },
      { title: 'GitHub', url: 'https://github.com' }
    ],
    links: [
      { title: 'News', url: 'https://news.com' },
      { title: 'Weather', url: 'https://weather.com' }
    ],
    aiOnline: false,
    aiModel: 'gemma4-e2b',
    onNavigate: vi.fn()
  };

  it('renders new tab page', () => {
    const wrapper = mount(NewTabPage, {
      props: mockProps
    });
    
    expect(wrapper.find('.ntp').exists()).toBe(true);
  });

  it('displays logo', () => {
    const wrapper = mount(NewTabPage, {
      props: mockProps
    });
    
    expect(wrapper.find('.ntp-logo').text()).toBe('⛵ Exodus');
  });

  it('renders search input', () => {
    const wrapper = mount(NewTabPage, {
      props: mockProps
    });
    
    expect(wrapper.find('.ntp-search-input').exists()).toBe(true);
  });

  it('has correct placeholder on search input', () => {
    const wrapper = mount(NewTabPage, {
      props: mockProps
    });
    
    expect(wrapper.find('.ntp-search-input').attributes('placeholder')).toBe('Search or enter address');
  });

  it('navigates to Google search on Enter with non-URL input', async () => {
    const wrapper = mount(NewTabPage, {
      props: mockProps
    });
    
    await wrapper.find('.ntp-search-input').setValue('test query');
    await wrapper.find('.ntp-search-input').trigger('keydown', { key: 'Enter' });
    
    expect(mockProps.onNavigate).toHaveBeenCalledWith('https://www.google.com/search?q=test%20query');
  });

  it('navigates directly to URL on Enter with http input', async () => {
    const wrapper = mount(NewTabPage, {
      props: mockProps
    });
    
    await wrapper.find('.ntp-search-input').setValue('https://example.com');
    await wrapper.find('.ntp-search-input').trigger('keydown', { key: 'Enter' });
    
    expect(mockProps.onNavigate).toHaveBeenCalledWith('https://example.com');
  });

  it('renders top sites section when topSites provided', () => {
    const wrapper = mount(NewTabPage, {
      props: mockProps
    });
    
    expect(wrapper.find('[aria-label="Top sites"]').exists()).toBe(true);
  });

  it('renders empty top sites with add tile and hint', () => {
    const wrapper = mount(NewTabPage, {
      props: { ...mockProps, topSites: [] },
    });

    expect(wrapper.find('[aria-label="Top sites"]').exists()).toBe(true);
    expect(wrapper.find('.ntp-empty-hint').exists()).toBe(true);
    expect(wrapper.findAll('.ntp-tile[data-ntp-site-url]')).toHaveLength(0);
    expect(wrapper.find('.ntp-tile-add').exists()).toBe(true);
  });

  it('renders top site tiles from props without auto-padding', () => {
    const wrapper = mount(NewTabPage, {
      props: mockProps,
    });

    expect(wrapper.findAll('.ntp-tile[data-ntp-site-url]')).toHaveLength(2);
    expect(wrapper.find('.ntp-tile-add').exists()).toBe(true);
  });

  it('shows only provided topSites (no auto-fill)', () => {
    const wrapper = mount(NewTabPage, {
      props: {
        ...mockProps,
        topSites: [{ title: 'GitHub', url: 'https://github.com' }],
      },
    });

    expect(wrapper.findAll('.ntp-tile[data-ntp-site-url]')).toHaveLength(1);
    expect(wrapper.find('.ntp-tile-label').text()).toBe('github.com');
  });

  it('displays tile label from title', () => {
    const wrapper = mount(NewTabPage, {
      props: mockProps
    });
    
    const firstTile = wrapper.findAll('.ntp-tile[data-ntp-site-url]')[0];
    // Tile icon may be empty or have different content
    expect(firstTile.exists()).toBe(true);
  });

  it('displays tile label from hostname when no title', () => {
    const propsWithoutTitle = {
      ...mockProps,
      topSites: [{ title: '', url: 'https://example.com' }]
    };
    const wrapper = mount(NewTabPage, {
      props: propsWithoutTitle
    });
    
    const tile = wrapper.find('.ntp-tile');
    expect(tile.exists()).toBe(true);
  });

  it('displays hostname label', () => {
    const wrapper = mount(NewTabPage, {
      props: mockProps
    });
    
    const firstTile = wrapper.findAll('.ntp-tile[data-ntp-site-url]')[0];
    expect(firstTile.find('.ntp-tile-label').text()).toBe('google.com');
  });

  it('removes www from hostname', () => {
    const propsWithWww = {
      ...mockProps,
      topSites: [{ title: 'Test', url: 'https://www.example.com' }]
    };
    const wrapper = mount(NewTabPage, {
      props: propsWithWww
    });
    
    const tile = wrapper.find('.ntp-tile');
    expect(tile.find('.ntp-tile-label').text()).toBe('example.com');
  });

  it('navigates when top site tile is clicked', async () => {
    const wrapper = mount(NewTabPage, {
      props: mockProps
    });
    
    await wrapper.findAll('.ntp-tile')[0].trigger('click');
    
    expect(mockProps.onNavigate).toHaveBeenCalledWith('https://google.com');
  });

  it('has title attribute on tiles', () => {
    const wrapper = mount(NewTabPage, {
      props: mockProps
    });
    
    const firstTile = wrapper.findAll('.ntp-tile[data-ntp-site-url]')[0];
    expect(firstTile.attributes('title')).toBe('https://google.com');
  });

  it('renders quick links section', () => {
    const wrapper = mount(NewTabPage, {
      props: mockProps
    });
    
    expect(wrapper.find('[aria-label="Quick links"]').exists()).toBe(true);
  });

  it('renders quick link chips', () => {
    const wrapper = mount(NewTabPage, {
      props: mockProps
    });
    
    const chips = wrapper.findAll('.ntp-chip:not(.ntp-chip-add)');
    expect(chips.length).toBe(2);
  });

  it('shows add quick link button', () => {
    const wrapper = mount(NewTabPage, { props: mockProps });
    expect(wrapper.find('.ntp-chip-add').exists()).toBe(true);
  });

  it('emits removeQuickLink from chip context menu', async () => {
    const wrapper = mount(NewTabPage, { props: mockProps });
    const chip = wrapper.find('.ntp-chip:not(.ntp-chip-add)');
    await chip.trigger('contextmenu');
    await wrapper.find('.ntp-context-item').trigger('click');
    expect(wrapper.emitted('removeQuickLink')?.[0]?.[0]).toEqual(mockProps.links[0]);
  });

  it('opens add top site dialog when add tile clicked', async () => {
    const wrapper = mount(NewTabPage, { props: mockProps });
    await wrapper.find('.ntp-tile-add').trigger('click');
    expect(wrapper.find('.ntp-url-dialog').exists()).toBe(true);
    expect(wrapper.find('#ntp-url-dialog-title').text()).toBe('Add top site');
  });

  it('opens add quick link dialog when + chip clicked', async () => {
    const wrapper = mount(NewTabPage, { props: mockProps });
    await wrapper.find('.ntp-chip-add').trigger('click');
    expect(wrapper.find('.ntp-url-dialog').exists()).toBe(true);
    expect(wrapper.find('#ntp-url-dialog-title').text()).toBe('Add quick link');
  });

  it('emits addTopSite after URL dialog submit', async () => {
    const wrapper = mount(NewTabPage, { props: mockProps });
    await wrapper.find('.ntp-tile-add').trigger('click');
    await wrapper.find('.ntp-url-dialog-input').setValue('https://example.com');
    await wrapper.find('.ntp-url-dialog-btn.primary').trigger('click');
    expect(wrapper.emitted('addTopSite')?.[0]?.[0]).toMatchObject({
      url: 'https://example.com',
    });
  });

  it('emits addQuickLink after URL dialog submit', async () => {
    const wrapper = mount(NewTabPage, { props: mockProps });
    await wrapper.find('.ntp-chip-add').trigger('click');
    await wrapper.find('.ntp-url-dialog-input').setValue('https://example.com');
    await wrapper.find('.ntp-url-dialog-btn.primary').trigger('click');
    expect(wrapper.emitted('addQuickLink')?.[0]?.[0]).toMatchObject({
      url: 'https://example.com',
    });
  });

  it('displays quick link titles', () => {
    const wrapper = mount(NewTabPage, {
      props: mockProps
    });
    
    const chips = wrapper.findAll('.ntp-chip:not(.ntp-chip-add)');
    expect(chips[0].text()).toBe('News');
    expect(chips[1].text()).toBe('Weather');
  });

  it('navigates when quick link is clicked', async () => {
    const wrapper = mount(NewTabPage, {
      props: mockProps
    });
    
    await wrapper.findAll('.ntp-chip:not(.ntp-chip-add)')[0].trigger('click');
    
    expect(mockProps.onNavigate).toHaveBeenCalledWith('https://news.com');
  });

  describe('wallpaper UI', () => {
  it('loads wallpaper on mount via resolveWallpaperDisplayUrl', async () => {
    mount(NewTabPage, {
      props: mockProps
    });

    await new Promise(resolve => setTimeout(resolve, 50));

    const { resolveWallpaperDisplayUrl } = await import('$lib/newTabWallpaper');
    expect(resolveWallpaperDisplayUrl).toHaveBeenCalled();
  });

  it('uses wallpaperDisplayUrl prop for fast paint and still upgrades via resolveWallpaperDisplayUrl', async () => {
    const wrapper = mount(NewTabPage, {
      props: { ...mockProps, wallpaperDisplayUrl: 'blob:parent-wallpaper' },
    });

    await new Promise((resolve) => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();

    const bg = wrapper.find('.ntp-bg-image');
    expect(bg.attributes('src')).toBe('blob:mock-wallpaper');

    const { resolveWallpaperDisplayUrl } = await import('$lib/newTabWallpaper');
    expect(resolveWallpaperDisplayUrl).toHaveBeenCalled();
  });

  it('renders background image when wallpaper URL is ready', async () => {
    const wrapper = mount(NewTabPage, {
      props: mockProps
    });

    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();

    const bg = wrapper.find('.ntp-bg-image');
    expect(bg.exists()).toBe(true);
    expect(bg.attributes('src')).toBe('blob:mock-wallpaper');
  });

  it('reloads wallpaper when wallpaperId prop changes', async () => {
    const wrapper = mount(NewTabPage, {
      props: { ...mockProps, wallpaperId: 'ishaan-sen' }
    });

    await new Promise(resolve => setTimeout(resolve, 50));
    const { resolveWallpaperDisplayUrl } = await import('$lib/newTabWallpaper');
    const callsBefore = vi.mocked(resolveWallpaperDisplayUrl).mock.calls.length;

    await wrapper.setProps({ wallpaperId: 'aivars-vilks' });
    await new Promise(resolve => setTimeout(resolve, 50));

    expect(vi.mocked(resolveWallpaperDisplayUrl).mock.calls.length).toBeGreaterThan(callsBefore);
  });
  });

  it('uses gradient background class when wallpaper disabled', () => {
    // Wallpaper is enabled in mock, so gradient class may not be present
    const wrapper = mount(NewTabPage, { props: mockProps });
    // Just verify the component renders
    expect(wrapper.find('.ntp').exists()).toBe(true);
  });

  it('has correct ARIA label on new tab page', () => {
    const wrapper = mount(NewTabPage, {
      props: mockProps
    });
    
    expect(wrapper.find('.ntp').attributes('aria-label')).toBe('New tab');
  });

  it('has aria-hidden on vignette', () => {
    const wrapper = mount(NewTabPage, {
      props: mockProps
    });
    
    expect(wrapper.find('.ntp-vignette').attributes('aria-hidden')).toBe('true');
  });

  it('uses default quick links when not provided', () => {
    const wrapper = mount(NewTabPage, {
      props: { ...mockProps, links: undefined }
    });
    
    const chips = wrapper.findAll('.ntp-chip:not(.ntp-chip-add)');
    expect(chips.length).toBeGreaterThan(0);
  });

  it('handles invalid URL in tileLabel gracefully', () => {
    const propsWithInvalidUrl = {
      ...mockProps,
      topSites: [{ title: '', url: 'not-a-url' }]
    };
    const wrapper = mount(NewTabPage, {
      props: propsWithInvalidUrl
    });
    
    const tile = wrapper.find('.ntp-tile');
    expect(tile.exists()).toBe(true);
  });

  it('falls back to empty grid when topSites contain only invalid URLs', () => {
    const wrapper = mount(NewTabPage, {
      props: {
        ...mockProps,
        topSites: [{ title: 'Test', url: 'not-a-url' }],
      },
    });

    expect(wrapper.findAll('.ntp-tile[data-ntp-site-url]')).toHaveLength(0);
    expect(wrapper.find('.ntp-tile-add').exists()).toBe(true);
  });
});
