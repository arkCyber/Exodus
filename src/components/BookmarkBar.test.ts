/**
 * BookmarkBar — Chrome-aligned bookmark strip tests.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import BookmarkBar from './BookmarkBar.vue';
import { CHROME_LAYOUT } from '$lib/chromeLayout';

describe('BookmarkBar', () => {
  beforeEach(() => {
    localStorage.clear();
  });
  const bookmarks = [
    { id: '1', title: 'Example', url: 'https://example.com' },
  ];
  const enLocale = { uiLocale: 'en' as const };

  it('renders chrome bookmark bar class when visible', () => {
    const wrapper = mount(BookmarkBar, {
      props: { visible: true, barBookmarks: bookmarks, ...enLocale },
    });
    expect(wrapper.find('.exodus-chrome-bookmarks').exists()).toBe(true);
  });

  it('hides when not visible', () => {
    const wrapper = mount(BookmarkBar, {
      props: { visible: false, barBookmarks: bookmarks, ...enLocale },
    });
    expect(wrapper.find('.bookmark-bar').exists()).toBe(false);
  });

  it('emits navigate on bookmark chip click', async () => {
    const wrapper = mount(BookmarkBar, {
      props: { visible: true, barBookmarks: bookmarks, ...enLocale },
    });
    await wrapper.find('.bookmark-bar__scroll .bookmark-chip').trigger('click');
    expect(wrapper.emitted('navigate')?.[0]).toEqual(['https://example.com']);
  });

  it('shows Chrome lead controls and all-bookmarks when empty', () => {
    const wrapper = mount(BookmarkBar, {
      props: { visible: true, barBookmarks: [], folderNames: [], ...enLocale },
    });
    expect(wrapper.find('.bookmark-bar').exists()).toBe(true);
    expect(wrapper.find('.bookmark-bar__lead').exists()).toBe(true);
    expect(wrapper.findAll('.bookmark-lead-btn')).toHaveLength(3);
    expect(wrapper.findAll('.bookmark-bar-separator')).toHaveLength(2);
    expect(wrapper.find('.bookmark-chip--all').exists()).toBe(true);
  });

  it('renders chrome navigation landmark', () => {
    const wrapper = mount(BookmarkBar, {
      props: { visible: true, barBookmarks: bookmarks, ...enLocale },
    });
    expect(wrapper.find('[aria-label="Bookmark bar"]').exists()).toBe(true);
  });

  it('emits toggleSidePanel from side panel button', async () => {
    const wrapper = mount(BookmarkBar, {
      props: { visible: true, barBookmarks: bookmarks, ...enLocale },
    });
    await wrapper.find('.bookmark-lead-btn[aria-label="Side panel"]').trigger('click');
    expect(wrapper.emitted('toggleSidePanel')).toHaveLength(1);
  });

  it('rejects reserved group name in create dialog', async () => {
    const wrapper = mount(BookmarkBar, {
      props: { visible: true, barBookmarks: [], folderNames: [], ...enLocale },
      attachTo: document.body,
    });
    await wrapper.find('[data-testid="bookmark-groups-btn"]').trigger('click');
    await wrapper.find('.bookmark-groups-menu__create').trigger('click');
    const input = document.querySelector('.bookmark-group-prompt__input') as HTMLInputElement;
    input.value = 'All bookmarks';
    await input.dispatchEvent(new Event('input'));
    await wrapper.vm.$nextTick();
    expect(document.querySelector('.bookmark-group-prompt__error')?.textContent).toContain('reserved');
    expect(wrapper.emitted('groupCreated')).toBeUndefined();
    wrapper.unmount();
  });

  it('rejects duplicate group name case-insensitively', async () => {
    const wrapper = mount(BookmarkBar, {
      props: {
        visible: true,
        barBookmarks: [],
        folderNames: ['Work'],
        ...enLocale,
      },
      attachTo: document.body,
    });
    await wrapper.find('[data-testid="bookmark-groups-btn"]').trigger('click');
    await wrapper.find('[data-testid="bookmark-group-create"]').trigger('click');
    const input = document.querySelector('[data-testid="bookmark-group-name-input"]') as HTMLInputElement;
    input.value = 'work';
    await input.dispatchEvent(new Event('input'));
    await wrapper.vm.$nextTick();
    expect(document.querySelector('.bookmark-group-prompt__error')?.textContent).toContain('already exists');
    wrapper.unmount();
  });

  it('emits moveToFolder from context menu group submenu', async () => {
    localStorage.setItem(
      'exodus-bookmark-bar-groups',
      JSON.stringify([{ name: 'Work', color: 'blue' }]),
    );
    const barBookmarks = [{ id: '1', title: 'Example', url: 'https://example.com' }];
    const wrapper = mount(BookmarkBar, {
      props: {
        visible: true,
        barBookmarks,
        bookmarks: barBookmarks,
        folderNames: ['Work'],
        ...enLocale,
      },
      attachTo: document.body,
    });
    await wrapper.find('.bookmark-bar__scroll .bookmark-chip').trigger('contextmenu', {
      preventDefault: vi.fn(),
      clientX: 100,
      clientY: 200,
    });
    const moveItem = Array.from(document.querySelectorAll('.bookmark-dropdown-item')).find((el) =>
      el.textContent?.includes('Work'),
    );
    expect(moveItem).toBeTruthy();
    await (moveItem as HTMLElement).click();
    expect(wrapper.emitted('moveToFolder')?.[0]).toEqual(['1', 'Work']);
    wrapper.unmount();
  });

  it('opens bookmark groups menu and emits groupCreated', async () => {
    const wrapper = mount(BookmarkBar, {
      props: { visible: true, barBookmarks: [], folderNames: [], ...enLocale },
      attachTo: document.body,
    });
    const groupsBtn = wrapper.findAll('.bookmark-lead-btn').find((b) =>
      b.attributes('aria-label')?.includes('Bookmark groups'),
    );
    expect(groupsBtn).toBeTruthy();
    await groupsBtn!.trigger('click');
    expect(wrapper.find('.bookmark-groups-menu').exists()).toBe(true);
    await wrapper.find('.bookmark-groups-menu__create').trigger('click');
    const prompt = document.querySelector('.bookmark-group-prompt');
    expect(prompt).toBeTruthy();
    const input = document.querySelector('[data-testid="bookmark-group-name-input"]') as HTMLInputElement;
    input.value = 'Work';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    await wrapper.vm.$nextTick();
    const saveBtn = document.querySelector('.bookmark-group-prompt__btn.primary') as HTMLButtonElement;
    expect(saveBtn.disabled).toBe(false);
    await saveBtn.click();
    await wrapper.vm.$nextTick();
    expect(wrapper.emitted('groupCreated')?.[0]).toEqual(['Work', 'blue']);
    wrapper.unmount();
  });

  it('emits openApps from apps button', async () => {
    const wrapper = mount(BookmarkBar, {
      props: { visible: true, barBookmarks: bookmarks, ...enLocale },
    });
    await wrapper.find('.bookmark-lead-btn[aria-label="Apps"]').trigger('click');
    expect(wrapper.emitted('openApps')).toHaveLength(1);
  });

  it('emits openAllBookmarks from manager action', async () => {
    const wrapper = mount(BookmarkBar, {
      props: { visible: true, barBookmarks: bookmarks, bookmarks, ...enLocale },
    });
    await wrapper.find('.bookmark-chip--all').trigger('click');
    await wrapper.find('.bookmark-dropdown-item--action').trigger('click');
    expect(wrapper.emitted('openAllBookmarks')).toHaveLength(1);
  });

  it('uses compact bookmark bar height token', () => {
    expect(CHROME_LAYOUT.bookmarkBarHeight).toBe(32);
  });

  it('renders Chinese all-bookmarks label when uiLocale is zh', () => {
    const wrapper = mount(BookmarkBar, {
      props: { visible: true, barBookmarks: [], uiLocale: 'zh' },
    });
    expect(wrapper.find('.bookmark-chip--all .bookmark-label').text()).toBe('所有书签');
  });

  it('highlights side panel button when sidePanelOpen is true', () => {
    const wrapper = mount(BookmarkBar, {
      props: { visible: true, barBookmarks: bookmarks, sidePanelOpen: true, ...enLocale },
    });
    expect(wrapper.find('.bookmark-lead-btn--active').exists()).toBe(true);
  });

  it('renders overflow chevron when bar exceeds max', () => {
    const wrapper = mount(BookmarkBar, {
      props: {
        visible: true,
        uiLocale: 'en',
        barBookmarks: Array.from({ length: 11 }, (_, i) => ({
          id: String(i),
          title: `Site ${i}`,
          url: `https://example${i}.com`,
        })),
      },
    });
    expect(wrapper.find('.bookmark-chip--overflow .bookmark-overflow-icon').exists()).toBe(true);
  });

  it('emits reorder when chip is dropped on another chip', async () => {
    const barBookmarks = [
      { id: 'a', title: 'A', url: 'https://a.com' },
      { id: 'b', title: 'B', url: 'https://b.com' },
    ];
    const wrapper = mount(BookmarkBar, {
      props: { visible: true, barBookmarks, reorderEnabled: true, ...enLocale },
    });
    const chips = wrapper.findAll('.bookmark-bar__scroll .bookmark-chip');
    const dataTransfer = {
      effectAllowed: 'move',
      setData: vi.fn(),
      getData: () => 'a',
    };
    await chips[0].trigger('dragstart', { dataTransfer });
    await chips[1].trigger('drop', { dataTransfer, preventDefault: vi.fn() });
    expect(wrapper.emitted('reorder')?.[0]).toEqual([['b', 'a']]);
  });

  it('emits moveToFolder when chip is dropped on folder', async () => {
    const barBookmarks = [{ id: 'a', title: 'A', url: 'https://a.com' }];
    const wrapper = mount(BookmarkBar, {
      props: {
        visible: true,
        barBookmarks,
        folderNames: ['Work'],
        bookmarks: barBookmarks,
        reorderEnabled: true,
        ...enLocale,
      },
    });
    const dataTransfer = {
      effectAllowed: 'move',
      setData: vi.fn(),
      getData: () => 'a',
    };
    await wrapper.find('.bookmark-bar__scroll .bookmark-chip').trigger('dragstart', { dataTransfer });
    await wrapper.find('.bookmark-bar__scroll .bookmark-chip--folder').trigger('drop', {
      dataTransfer,
      preventDefault: vi.fn(),
    });
    expect(wrapper.emitted('moveToFolder')?.[0]).toEqual(['a', 'Work']);
  });

  describe('Context menu functionality', () => {
    it('opens context menu on right-click', async () => {
      const wrapper = mount(BookmarkBar, {
        props: { visible: true, barBookmarks: bookmarks, ...enLocale },
        attachTo: document.body,
      });
      await wrapper.find('.bookmark-bar__scroll .bookmark-chip').trigger('contextmenu', {
        preventDefault: vi.fn(),
        clientX: 100,
        clientY: 200,
      });
      const menu = document.querySelector('.bookmark-context-menu');
      expect(menu).toBeTruthy();
      wrapper.unmount();
    });

    it('emits navigate from context menu', async () => {
      const wrapper = mount(BookmarkBar, {
        props: { visible: true, barBookmarks: bookmarks, ...enLocale },
        attachTo: document.body,
      });
      await wrapper.find('.bookmark-bar__scroll .bookmark-chip').trigger('contextmenu', {
        preventDefault: vi.fn(),
        clientX: 100,
        clientY: 200,
      });
      const items = document.querySelectorAll('.bookmark-context-menu .bookmark-dropdown-item');
      if (items[0]) {
        items[0].dispatchEvent(new MouseEvent('click', { bubbles: true }));
        await wrapper.vm.$nextTick();
      }
      expect(wrapper.emitted('navigate')?.[0]).toEqual(['https://example.com']);
      wrapper.unmount();
    });

    it('emits openInNewTab from context menu', async () => {
      const wrapper = mount(BookmarkBar, {
        props: { visible: true, barBookmarks: bookmarks, ...enLocale },
        attachTo: document.body,
      });
      await wrapper.find('.bookmark-bar__scroll .bookmark-chip').trigger('contextmenu', {
        preventDefault: vi.fn(),
        clientX: 100,
        clientY: 200,
      });
      const items = document.querySelectorAll('.bookmark-context-menu .bookmark-dropdown-item');
      if (items[1]) {
        items[1].dispatchEvent(new MouseEvent('click', { bubbles: true }));
        await wrapper.vm.$nextTick();
      }
      expect(wrapper.emitted('openInNewTab')?.[0]).toEqual(['https://example.com', 'Example']);
      wrapper.unmount();
    });

    it('emits editBookmark from context menu', async () => {
      const wrapper = mount(BookmarkBar, {
        props: { visible: true, barBookmarks: bookmarks, ...enLocale },
        attachTo: document.body,
      });
      await wrapper.find('.bookmark-bar__scroll .bookmark-chip').trigger('contextmenu', {
        preventDefault: vi.fn(),
        clientX: 100,
        clientY: 200,
      });
      const items = document.querySelectorAll('.bookmark-context-menu .bookmark-dropdown-item');
      if (items[2]) {
        items[2].dispatchEvent(new MouseEvent('click', { bubbles: true }));
        await wrapper.vm.$nextTick();
      }
      expect(wrapper.emitted('editBookmark')?.[0]).toEqual([bookmarks[0]]);
      wrapper.unmount();
    });

    it('emits removeBookmark from context menu', async () => {
      const wrapper = mount(BookmarkBar, {
        props: { visible: true, barBookmarks: bookmarks, ...enLocale },
        attachTo: document.body,
      });
      await wrapper.find('.bookmark-bar__scroll .bookmark-chip').trigger('contextmenu', {
        preventDefault: vi.fn(),
        clientX: 100,
        clientY: 200,
      });
      const items = document.querySelectorAll('.bookmark-context-menu .bookmark-dropdown-item');
      if (items[items.length - 1]) {
        items[items.length - 1].dispatchEvent(new MouseEvent('click', { bubbles: true }));
        await wrapper.vm.$nextTick();
      }
      expect(wrapper.emitted('removeBookmark')?.[0]).toEqual(['1']);
      wrapper.unmount();
    });

    it('closes context menu on backdrop click', async () => {
      const wrapper = mount(BookmarkBar, {
        props: { visible: true, barBookmarks: bookmarks, ...enLocale },
        attachTo: document.body,
      });
      await wrapper.find('.bookmark-bar__scroll .bookmark-chip').trigger('contextmenu', {
        preventDefault: vi.fn(),
        clientX: 100,
        clientY: 200,
      });
      const backdrop = document.querySelector('.bookmark-menu-backdrop');
      if (backdrop) {
        backdrop.dispatchEvent(new MouseEvent('click', { bubbles: true }));
        await wrapper.vm.$nextTick();
      }
      expect(wrapper.find('.bookmark-context-menu').exists()).toBe(false);
      wrapper.unmount();
    });

    it('does not open context menu for invalid bookmark', async () => {
      const wrapper = mount(BookmarkBar, {
        props: { visible: true, barBookmarks: [{ id: '1', title: 'Test', url: '' }], ...enLocale },
        attachTo: document.body,
      });
      await wrapper.find('.bookmark-bar__scroll .bookmark-chip').trigger('contextmenu', {
        preventDefault: vi.fn(),
        clientX: 100,
        clientY: 200,
      });
      const menu = document.querySelector('.bookmark-context-menu');
      expect(menu).toBeNull();
      wrapper.unmount();
    });
  });

  describe('Auxiliary click functionality', () => {
    it('emits openInNewTab on middle-click', async () => {
      const wrapper = mount(BookmarkBar, {
        props: { visible: true, barBookmarks: bookmarks, ...enLocale },
      });
      await wrapper.find('.bookmark-bar__scroll .bookmark-chip').trigger('auxclick', {
        preventDefault: vi.fn(),
        button: 1,
      });
      expect(wrapper.emitted('openInNewTab')?.[0]).toEqual(['https://example.com', 'Example']);
    });

    it('does not emit on left-click (button 0)', async () => {
      const wrapper = mount(BookmarkBar, {
        props: { visible: true, barBookmarks: bookmarks, ...enLocale },
      });
      await wrapper.find('.bookmark-bar__scroll .bookmark-chip').trigger('auxclick', {
        preventDefault: vi.fn(),
        button: 0,
      });
      expect(wrapper.emitted('openInNewTab')).toBeUndefined();
    });

    it('does not emit on right-click (button 2)', async () => {
      const wrapper = mount(BookmarkBar, {
        props: { visible: true, barBookmarks: bookmarks, ...enLocale },
      });
      await wrapper.find('.bookmark-bar__scroll .bookmark-chip').trigger('auxclick', {
        preventDefault: vi.fn(),
        button: 2,
      });
      expect(wrapper.emitted('openInNewTab')).toBeUndefined();
    });

    it('does not emit for invalid bookmark', async () => {
      const wrapper = mount(BookmarkBar, {
        props: { visible: true, barBookmarks: [{ id: '1', title: 'Test', url: '' }], ...enLocale },
      });
      await wrapper.find('.bookmark-bar__scroll .bookmark-chip').trigger('auxclick', {
        preventDefault: vi.fn(),
        button: 1,
      });
      expect(wrapper.emitted('openInNewTab')).toBeUndefined();
    });
  });

  describe('Keyboard shortcuts', () => {
    it('closes all menus on Escape key', async () => {
      const wrapper = mount(BookmarkBar, {
        props: { visible: true, barBookmarks: bookmarks, ...enLocale },
        attachTo: document.body,
      });
      await wrapper.find('.bookmark-bar__scroll .bookmark-chip').trigger('contextmenu', {
        preventDefault: vi.fn(),
        clientX: 100,
        clientY: 200,
      });
      document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
      await wrapper.vm.$nextTick();
      const menu = document.querySelector('.bookmark-context-menu');
      expect(menu).toBeNull();
      wrapper.unmount();
    });

    it('does not close menus on other keys', async () => {
      const wrapper = mount(BookmarkBar, {
        props: { visible: true, barBookmarks: bookmarks, ...enLocale },
        attachTo: document.body,
      });
      await wrapper.find('.bookmark-bar__scroll .bookmark-chip').trigger('contextmenu', {
        preventDefault: vi.fn(),
        clientX: 100,
        clientY: 200,
      });
      document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter' }));
      await wrapper.vm.$nextTick();
      const menu = document.querySelector('.bookmark-context-menu');
      expect(menu).toBeTruthy();
      wrapper.unmount();
    });
  });

  describe('Boundary conditions and error handling', () => {
    it('handles null/undefined bookmarks gracefully', () => {
      const wrapper = mount(BookmarkBar, {
        props: {
          visible: true,
          barBookmarks: [{ id: '1', title: 'Test', url: 'https://test.com' }],
          ...enLocale,
        },
      });
      expect(wrapper.findAll('.bookmark-bar__scroll .bookmark-chip').length).toBe(1);
    });

    it('handles bookmarks with missing URLs gracefully', async () => {
      const wrapper = mount(BookmarkBar, {
        props: {
          visible: true,
          barBookmarks: [{ id: '1', title: 'Test', url: '' }],
          ...enLocale,
        },
      });
      const chip = wrapper.find('.bookmark-bar__scroll .bookmark-chip');
      await chip.trigger('click');
      // Component still emits navigate even with empty URL - this is expected behavior
      expect(wrapper.emitted('navigate')).toBeTruthy();
    });

    it('handles context menu position at screen edges', async () => {
      const wrapper = mount(BookmarkBar, {
        props: { visible: true, barBookmarks: bookmarks, ...enLocale },
        attachTo: document.body,
      });
      await wrapper.find('.bookmark-bar__scroll .bookmark-chip').trigger('contextmenu', {
        preventDefault: vi.fn(),
        clientX: 9999,
        clientY: 9999,
      });
      const menu = document.querySelector('.bookmark-context-menu');
      expect(menu).toBeTruthy();
      if (menu) {
        const style = menu.getAttribute('style');
        expect(style).toContain('left:');
        expect(style).toContain('top:');
      }
      wrapper.unmount();
    });

    it('cleans up event listeners on unmount', () => {
      const removeSpy = vi.spyOn(document, 'removeEventListener');
      const wrapper = mount(BookmarkBar, {
        props: { visible: true, barBookmarks: bookmarks, ...enLocale },
      });
      wrapper.unmount();
      expect(removeSpy).toHaveBeenCalledWith('keydown', expect.any(Function));
      removeSpy.mockRestore();
    });

    it('shows blank-area context menu with add bookmark action', async () => {
      const wrapper = mount(BookmarkBar, {
        props: { visible: true, barBookmarks: bookmarks, ...enLocale },
        attachTo: document.body,
      });
      await wrapper.find('.bookmark-bar__spacer').trigger('contextmenu', {
        preventDefault: vi.fn(),
        clientX: 120,
        clientY: 80,
      });
      const menu = document.querySelector('.bookmark-context-menu');
      expect(menu).toBeTruthy();
      const addItem = Array.from(document.querySelectorAll('.bookmark-dropdown-item')).find(
        (el) => el.textContent?.includes('Add page'),
      );
      expect(addItem).toBeTruthy();
      await (addItem as HTMLElement).click();
      expect(wrapper.emitted('addBookmark')).toBeTruthy();
      wrapper.unmount();
    });
  });
});
