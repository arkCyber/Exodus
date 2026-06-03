/**
 * Exodus Browser — Settings persistence integration tests.
 * Tests that all settings components properly persist to localStorage.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import SearchEngineSettings from './SearchEngineSettings.vue';
import FontAndZoomSettings from './FontAndZoomSettings.vue';
import NetworkSettings from './NetworkSettings.vue';
import MediaSettings from './MediaSettings.vue';
import NotificationSettings from './NotificationSettings.vue';
import AccessibilitySettings from './AccessibilitySettings.vue';
import KeyboardShortcutsSettings from './KeyboardShortcutsSettings.vue';
import SystemSettings from './SystemSettings.vue';

describe('Settings Persistence Integration', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('SearchEngineSettings persists to localStorage', async () => {
    const wrapper = mount(SearchEngineSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    
    const select = wrapper.find('[data-testid="search-engine-default"]');
    await select.setValue('google');
    await flushPromises();
    
    const stored = localStorage.getItem('exodus-default-search-engine');
    expect(stored).toBe('google');
  });

  it('FontAndZoomSettings persists to localStorage', async () => {
    const wrapper = mount(FontAndZoomSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    
    const zoomSelect = wrapper.find('[data-testid="default-zoom"]');
    await zoomSelect.setValue('125');
    await flushPromises();
    
    const stored = localStorage.getItem('exodus-font-zoom-settings');
    expect(stored).toBeTruthy();
    const data = JSON.parse(stored!);
    expect(data.defaultZoom).toBe('125');
  });

  it('NetworkSettings persists to localStorage', async () => {
    const wrapper = mount(NetworkSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    
    const checkbox = wrapper.find('input[type="checkbox"]');
    await checkbox.setChecked(true);
    await flushPromises();
    
    const stored = localStorage.getItem('exodus-network-settings');
    expect(stored).toBeTruthy();
    const data = JSON.parse(stored!);
    expect(data.proxyEnabled).toBe(true);
  });

  it('MediaSettings persists to localStorage', async () => {
    const wrapper = mount(MediaSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    
    const select = wrapper.find('select');
    await select.setValue('block');
    await flushPromises();
    
    const stored = localStorage.getItem('exodus-media-settings');
    expect(stored).toBeTruthy();
    const data = JSON.parse(stored!);
    expect(data.autoplayPolicy).toBe('block');
  });

  it('NotificationSettings persists to localStorage', async () => {
    const wrapper = mount(NotificationSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    
    const checkbox = wrapper.find('input[type="checkbox"]');
    await checkbox.setChecked(false);
    await flushPromises();
    
    const stored = localStorage.getItem('exodus-notification-settings');
    expect(stored).toBeTruthy();
    const data = JSON.parse(stored!);
    expect(data.notificationsEnabled).toBe(false);
  });

  it('AccessibilitySettings persists to localStorage', async () => {
    const wrapper = mount(AccessibilitySettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    
    const checkbox = wrapper.find('input[type="checkbox"]');
    await checkbox.setChecked(true);
    await flushPromises();
    
    const stored = localStorage.getItem('exodus-accessibility-settings');
    expect(stored).toBeTruthy();
    const data = JSON.parse(stored!);
    expect(data.forceDarkMode).toBe(true);
  });

  it('KeyboardShortcutsSettings persists custom shortcuts to localStorage', async () => {
    const wrapper = mount(KeyboardShortcutsSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    
    const addButton = wrapper.findAll('button').find(b => b.text().includes('Add'));
    if (addButton) {
      await addButton.trigger('click');
      await flushPromises();
      
      const stored = localStorage.getItem('exodus-keyboard-shortcuts');
      expect(stored).toBeTruthy();
      const data = JSON.parse(stored!);
      expect(data.custom).toBeDefined();
    }
  });

  it('SystemSettings persists to localStorage', async () => {
    const wrapper = mount(SystemSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    
    const checkbox = wrapper.find('input[type="checkbox"]');
    await checkbox.setChecked(true);
    await flushPromises();
    
    const stored = localStorage.getItem('exodus-system-settings');
    expect(stored).toBeTruthy();
    const data = JSON.parse(stored!);
    expect(data.defaultBrowser).toBe(true);
  });

  it('FontAndZoomSettings loads from localStorage', async () => {
    localStorage.setItem('exodus-font-zoom-settings', JSON.stringify({
      defaultZoom: '150',
      standardFont: 'Arial',
      serifFont: 'Times New Roman',
      sansSerifFont: 'Helvetica',
      monospaceFont: 'Courier New',
      fontSize: 16,
      smoothScrolling: true
    }));
    
    const wrapper = mount(FontAndZoomSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    
    const zoomSelect = wrapper.find('[data-testid="default-zoom"]');
    expect(zoomSelect.element.value).toBe('150');
  });

  it('NetworkSettings loads from localStorage', async () => {
    localStorage.setItem('exodus-network-settings', JSON.stringify({
      proxyEnabled: true,
      proxyType: 'https',
      proxyHost: '127.0.0.1',
      proxyPort: 8080,
      proxyAuth: false,
      dnsOverHttps: true,
      dnsProvider: 'cloudflare',
      httpProtocol: 'https'
    }));
    
    const wrapper = mount(NetworkSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    
    const checkbox = wrapper.find('input[type="checkbox"]');
    expect(checkbox.element.checked).toBe(true);
  });

  it('MediaSettings loads from localStorage', async () => {
    localStorage.setItem('exodus-media-settings', JSON.stringify({
      autoplayPolicy: 'block',
      pipEnabled: false,
      pipAutoEnter: false,
      hardwareAcceleration: false,
      preloadMedia: true,
      defaultVolume: 50,
      castingEnabled: true
    }));
    
    const wrapper = mount(MediaSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    
    const select = wrapper.find('select');
    expect(select.element.value).toBe('block');
  });

  it('NotificationSettings loads from localStorage', async () => {
    localStorage.setItem('exodus-notification-settings', JSON.stringify({
      notificationsEnabled: false,
      defaultBehavior: 'block',
      soundEnabled: false,
      badgeEnabled: false,
      quietMode: true,
      quietStart: '23:00',
      quietEnd: '07:00'
    }));
    
    const wrapper = mount(NotificationSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    
    const checkbox = wrapper.find('input[type="checkbox"]');
    expect(checkbox.element.checked).toBe(false);
  });

  it('AccessibilitySettings loads from localStorage', async () => {
    localStorage.setItem('exodus-accessibility-settings', JSON.stringify({
      forceDarkMode: true,
      reduceMotion: true,
      highContrast: true,
      screenReader: true,
      minimumFontSize: 18,
      cursorSize: 'large',
      focusIndicator: false,
      textToSpeech: true
    }));
    
    const wrapper = mount(AccessibilitySettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    
    const checkbox = wrapper.find('input[type="checkbox"]');
    expect(checkbox.element.checked).toBe(true);
  });

  it('SystemSettings loads from localStorage', async () => {
    localStorage.setItem('exodus-system-settings', JSON.stringify({
      defaultBrowser: true,
      backgroundApps: false,
      hardwareAcceleration: false,
      useGPURendering: false,
      updateAutomatically: false,
      updateChannel: 'nightly'
    }));
    
    const wrapper = mount(SystemSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    
    const checkbox = wrapper.find('input[type="checkbox"]');
    expect(checkbox.element.checked).toBe(true);
  });

  it('FontAndZoomSettings resets to defaults', async () => {
    localStorage.setItem('exodus-font-zoom-settings', JSON.stringify({
      defaultZoom: '200',
      standardFont: 'Comic Sans MS',
      serifFont: 'Georgia',
      sansSerifFont: 'Verdana',
      monospaceFont: 'Consolas',
      fontSize: 24,
      smoothScrolling: false
    }));
    
    const wrapper = mount(FontAndZoomSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    
    const resetButton = wrapper.findAll('button').find(b => b.text().includes('Reset'));
    if (resetButton) {
      await resetButton.trigger('click');
      await flushPromises();
      
      const stored = localStorage.getItem('exodus-font-zoom-settings');
      const data = JSON.parse(stored!);
      expect(data.defaultZoom).toBe(100);
    }
  });

  it('NetworkSettings resets to defaults', async () => {
    localStorage.setItem('exodus-network-settings', JSON.stringify({
      proxyEnabled: true,
      proxyType: 'socks5',
      proxyHost: '192.168.1.1',
      proxyPort: 1080,
      proxyAuth: true,
      dnsOverHttps: true,
      dnsProvider: 'google',
      httpProtocol: 'https'
    }));
    
    const wrapper = mount(NetworkSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    
    const resetButton = wrapper.findAll('button').find(b => b.text().includes('Reset'));
    if (resetButton) {
      await resetButton.trigger('click');
      await flushPromises();
      
      const stored = localStorage.getItem('exodus-network-settings');
      const data = JSON.parse(stored!);
      expect(data.proxyEnabled).toBe(false);
    }
  });

  it('MediaSettings resets to defaults', async () => {
    localStorage.setItem('exodus-media-settings', JSON.stringify({
      autoplayPolicy: 'block',
      pipEnabled: false,
      pipAutoEnter: false,
      hardwareAcceleration: false,
      preloadMedia: false,
      defaultVolume: 0,
      castingEnabled: false
    }));
    
    const wrapper = mount(MediaSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    
    const resetButton = wrapper.findAll('button').find(b => b.text().includes('Reset'));
    if (resetButton) {
      await resetButton.trigger('click');
      await flushPromises();
      
      const stored = localStorage.getItem('exodus-media-settings');
      const data = JSON.parse(stored!);
      expect(data.autoplayPolicy).toBe('limit');
    }
  });

  it('NotificationSettings resets to defaults', async () => {
    localStorage.setItem('exodus-notification-settings', JSON.stringify({
      notificationsEnabled: false,
      defaultBehavior: 'block',
      soundEnabled: false,
      badgeEnabled: false,
      quietMode: true,
      quietStart: '23:00',
      quietEnd: '07:00'
    }));
    
    const wrapper = mount(NotificationSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    
    const resetButton = wrapper.findAll('button').find(b => b.text().includes('Reset'));
    if (resetButton) {
      await resetButton.trigger('click');
      await flushPromises();
      
      const stored = localStorage.getItem('exodus-notification-settings');
      const data = JSON.parse(stored!);
      expect(data.notificationsEnabled).toBe(true);
    }
  });

  it('AccessibilitySettings resets to defaults', async () => {
    localStorage.setItem('exodus-accessibility-settings', JSON.stringify({
      forceDarkMode: true,
      reduceMotion: true,
      highContrast: true,
      screenReader: true,
      minimumFontSize: 24,
      cursorSize: 'extra-large',
      focusIndicator: false,
      textToSpeech: true
    }));
    
    const wrapper = mount(AccessibilitySettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    
    const resetButton = wrapper.findAll('button').find(b => b.text().includes('Reset'));
    if (resetButton) {
      await resetButton.trigger('click');
      await flushPromises();
      
      const stored = localStorage.getItem('exodus-accessibility-settings');
      const data = JSON.parse(stored!);
      expect(data.forceDarkMode).toBe(false);
    }
  });

  it('SystemSettings resets to defaults', async () => {
    localStorage.setItem('exodus-system-settings', JSON.stringify({
      defaultBrowser: true,
      backgroundApps: false,
      hardwareAcceleration: false,
      useGPURendering: false,
      updateAutomatically: false,
      updateChannel: 'nightly'
    }));
    
    const wrapper = mount(SystemSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    
    const resetButton = wrapper.findAll('button').find(b => b.text().includes('Reset'));
    if (resetButton) {
      await resetButton.trigger('click');
      await flushPromises();
      
      const stored = localStorage.getItem('exodus-system-settings');
      const data = JSON.parse(stored!);
      expect(data.defaultBrowser).toBe(false);
    }
  });
});
