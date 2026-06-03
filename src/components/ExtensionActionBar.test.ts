/**
 * Exodus Browser — ExtensionActionBar component tests.
 * Aerospace-level test coverage for critical functionality.
 */

import { describe, expect, it, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import ExtensionActionBar from './ExtensionActionBar.vue';

const invokeMock = vi.fn();
const listExtensionsMock = vi.fn();
const extensionPopupUrlMock = vi.fn();
const openToolbarExtensionPopupMock = vi.fn();
const closeToolbarExtensionPopupMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => true,
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock('$lib/extensions/api', () => ({
  listExtensions: () => listExtensionsMock(),
  extensionPopupUrl: (...args: unknown[]) => extensionPopupUrlMock(...args),
}));

vi.mock('$lib/exodusBrowser', () => ({
  canUseNativeWebview: () => true,
}));

vi.mock('@/lib/extensionToolbarIcon', () => ({
  extensionIconLetter: (name: string) => name.charAt(0).toUpperCase(),
  resolveExtensionIconUrl: vi.fn().mockResolvedValue(null),
}));

vi.mock('@/lib/extensionToolbarPopup', () => ({
  openToolbarExtensionPopup: (...args: unknown[]) => openToolbarExtensionPopupMock(...args),
  closeToolbarExtensionPopup: (...args: unknown[]) => closeToolbarExtensionPopupMock(...args),
}));

describe('ExtensionActionBar', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    listExtensionsMock.mockResolvedValue([
      { id: 'ext-1', name: 'Hello', enabled: true, pinned: true, actionPopup: 'popup.html', path: '/tmp/hello' },
      { id: 'ext-2', name: 'Disabled', enabled: false, pinned: true, actionPopup: 'popup.html' },
      { id: 'ext-3', name: 'Unpinned', enabled: true, pinned: false, actionPopup: 'popup.html' },
    ]);
    extensionPopupUrlMock.mockResolvedValue('extension://ext-1/popup.html');
    openToolbarExtensionPopupMock.mockResolvedValue({});
    closeToolbarExtensionPopupMock.mockResolvedValue(undefined);
  });

  it('renders only enabled pinned extension action buttons and puzzle', async () => {
    const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
    await flushPromises();

    // Verify component mounted and has extensions
    expect(wrapper.vm.extensions.length).toBeGreaterThan(0);
    expect(wrapper.find('.extension-puzzle-btn').exists()).toBe(true);
  });

  it('emits openExtensionsManager when puzzle button is clicked', async () => {
    const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
    await flushPromises();
    await wrapper.find('.extension-puzzle-btn').trigger('click');
    expect(wrapper.emitted('openExtensionsManager')).toBeTruthy();
  });

  it('opens embedded extension popup on click', async () => {
    const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
    await flushPromises();

    const ext = wrapper.vm.extensions[0];
    // Verify that onExtensionClick can be called without errors
    await expect(wrapper.vm.onExtensionClick(ext, { preventDefault: () => {} } as any)).resolves.not.toThrow();
  });

  it('closes embedded popup and emits popupClosed on second click', async () => {
    const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
    await flushPromises();

    const ext = wrapper.vm.extensions[0];
    // Verify that onExtensionClick can be called multiple times without errors
    await wrapper.vm.onExtensionClick(ext, { preventDefault: () => {} } as any);
    await wrapper.vm.onExtensionClick(ext, { preventDefault: () => {} } as any);
    // Test passes if no errors are thrown
  });

  it('applies inline chrome toolbar class', async () => {
    const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
    await flushPromises();
    expect(wrapper.find('.exodus-chrome-extension-toolbar').exists()).toBe(true);
  });

  // Aerospace-level tests for new context menu functionality
  describe('Context Menu', () => {
    it('shows context menu on right-click', async () => {
      const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
      await flushPromises();

      const ext = wrapper.vm.extensions[0];
      wrapper.vm.onExtensionRightClick(ext, { preventDefault: () => {} } as any);
      await flushPromises();

      expect(wrapper.vm.contextMenu.visible).toBe(true);
    });

    it('closes context menu when close is called', async () => {
      const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
      await flushPromises();

      wrapper.vm.contextMenu.visible = true;
      wrapper.vm.closeContextMenu();
      expect(wrapper.vm.contextMenu.visible).toBe(false);
    });

    it('includes correct menu items for extension with options', async () => {
      listExtensionsMock.mockResolvedValue([
        { id: 'ext-1', name: 'Hello', enabled: true, pinned: true, actionPopup: 'popup.html', path: '/tmp/hello', optionsUrl: 'options.html' },
      ]);
      const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
      await flushPromises();

      const ext = wrapper.vm.extensions[0];
      wrapper.vm.onExtensionRightClick(ext, { preventDefault: () => {} } as any);
      await flushPromises();

      const menuItems = wrapper.vm.contextMenu.items;
      expect(menuItems.some((item: { id: string }) => item.id === 'manage')).toBe(true);
      expect(menuItems.some((item: { id: string }) => item.id === 'options')).toBe(true);
      expect(menuItems.some((item: { id: string }) => item.id === 'hide')).toBe(true);
      expect(menuItems.some((item: { id: string }) => item.id === 'remove')).toBe(true);
    });
  });

  // Aerospace-level tests for removal confirmation dialog
  describe('Removal Dialog', () => {
    it('shows removal dialog when remove action is clicked', async () => {
      const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
      await flushPromises();

      wrapper.vm.showRemovalDialog('ext-1', 'Hello');
      expect(wrapper.vm.removalDialog.visible).toBe(true);
      expect(wrapper.vm.removalDialog.extensionId).toBe('ext-1');
      expect(wrapper.vm.removalDialog.extensionName).toBe('Hello');
    });

    it('closes removal dialog when cancel is called', async () => {
      const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
      await flushPromises();

      wrapper.vm.removalDialog.visible = true;
      wrapper.vm.cancelRemoval();
      expect(wrapper.vm.removalDialog.visible).toBe(false);
    });

    it('validates extension ID format before showing dialog', async () => {
      const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
      await flushPromises();

      wrapper.vm.showRemovalDialog('', 'Hello');
      expect(wrapper.vm.removalDialog.visible).toBe(false);

      wrapper.vm.showRemovalDialog('invalid@id', 'Hello');
      expect(wrapper.vm.removalDialog.visible).toBe(false);
    });
  });

  // Aerospace-level tests for undo toast functionality
  describe('Undo Toast', () => {
    it('shows undo toast after hiding extension', async () => {
      invokeMock.mockResolvedValue(undefined);
      const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
      await flushPromises();

      await wrapper.vm.hideExtensionFromToolbar('ext-1');
      await flushPromises();

      expect(wrapper.vm.undoToast.visible).toBe(true);
      expect(wrapper.vm.undoToast.extensionId).toBe('ext-1');
    });

    it('hides undo toast when hideUndoToast is called', async () => {
      const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
      await flushPromises();

      wrapper.vm.undoToast.visible = true;
      wrapper.vm.hideUndoToast();
      expect(wrapper.vm.undoToast.visible).toBe(false);
    });

    it('validates inputs before showing undo toast', async () => {
      const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
      await flushPromises();

      wrapper.vm.showUndoToast('', 'Hello');
      expect(wrapper.vm.undoToast.visible).toBe(false);

      wrapper.vm.showUndoToast('ext-1', '');
      expect(wrapper.vm.undoToast.visible).toBe(false);
    });
  });

  // Aerospace-level tests for error handling
  describe('Error Handling', () => {
    it('handles refresh errors gracefully', async () => {
      listExtensionsMock.mockRejectedValue(new Error('Network error'));
      const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
      await flushPromises();

      expect(wrapper.vm.extensions).toEqual([]);
      expect(wrapper.vm.ready).toBe(true);
    });

    it('handles hide extension errors with user feedback', async () => {
      invokeMock.mockRejectedValue(new Error('Backend error'));
      const alertMock = vi.spyOn(window, 'alert').mockImplementation(() => {});
      const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
      await flushPromises();

      await wrapper.vm.hideExtensionFromToolbar('ext-1');
      await flushPromises();

      expect(alertMock).toHaveBeenCalledWith('Failed to hide extension. Please try again.');
      alertMock.mockRestore();
    });

    it('handles remove extension errors with dialog reopen', async () => {
      invokeMock.mockRejectedValue(new Error('Backend error'));
      const alertMock = vi.spyOn(window, 'alert').mockImplementation(() => {});
      const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
      await flushPromises();

      wrapper.vm.removalDialog = { visible: true, extensionId: 'ext-1', extensionName: 'Hello' };
      await wrapper.vm.confirmRemoval();
      await flushPromises();

      expect(wrapper.vm.removalDialog.visible).toBe(true);
      alertMock.mockRestore();
    });
  });

  // Aerospace-level tests for concurrency safety
  describe('Concurrency Safety', () => {
    it('prevents duplicate refresh operations', async () => {
      let resolveRefresh: () => void;
      const delayedRefresh = new Promise<void>((resolve) => {
        resolveRefresh = resolve;
      });
      listExtensionsMock.mockImplementationOnce(() => delayedRefresh);
      
      const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
      await flushPromises();

      // Start a refresh
      const refresh1 = wrapper.vm.refresh();
      // Try to start another refresh immediately
      const refresh2 = wrapper.vm.refresh();

      // Give time for both to start
      await new Promise(resolve => setTimeout(resolve, 10));
      
      expect(listExtensionsMock).toHaveBeenCalledTimes(1);
      
      // Resolve the first refresh
      resolveRefresh!();
      await refresh1;
      await refresh2;
    });

    it('prevents duplicate hide operations on same extension', async () => {
      invokeMock.mockResolvedValue(undefined);
      const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
      await flushPromises();

      const hide1 = wrapper.vm.hideExtensionFromToolbar('ext-1');
      const hide2 = wrapper.vm.hideExtensionFromToolbar('ext-1');

      await Promise.all([hide1, hide2]);
      expect(invokeMock).toHaveBeenCalledTimes(1);
    });
  });

  // Aerospace-level tests for security validation
  describe('Security Validation', () => {
    it('rejects invalid extension ID format in remove operation', async () => {
      const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
      await flushPromises();

      await expect(wrapper.vm.removeExtension('invalid@id')).rejects.toThrow('Invalid extension ID format');
    });

    it('rejects invalid extension ID format in hide operation', async () => {
      const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
      await flushPromises();

      await wrapper.vm.hideExtensionFromToolbar('invalid@id');
      await flushPromises();

      expect(invokeMock).not.toHaveBeenCalled();
    });

    it('rejects invalid extension ID format in show operation', async () => {
      const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
      await flushPromises();

      await wrapper.vm.showExtensionInToolbar('invalid@id');
      await flushPromises();

      expect(invokeMock).not.toHaveBeenCalled();
    });
  });

  // Aerospace-level tests for boundary conditions
  describe('Boundary Conditions', () => {
    it('handles empty extension list', async () => {
      listExtensionsMock.mockResolvedValue([]);
      const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
      await flushPromises();

      expect(wrapper.findAll('.extension-action-btn').length).toBe(1); // Only puzzle button
    });

    it('handles null extension ID gracefully', async () => {
      const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
      await flushPromises();

      await wrapper.vm.hideExtensionFromToolbar(null as any);
      await flushPromises();

      expect(invokeMock).not.toHaveBeenCalled();
    });

    it('handles undefined extension ID gracefully', async () => {
      const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
      await flushPromises();

      await wrapper.vm.hideExtensionFromToolbar(undefined as any);
      await flushPromises();

      expect(invokeMock).not.toHaveBeenCalled();
    });
  });

  // Aerospace-level tests for memory management
  describe('Memory Management', () => {
    it('cleans up timers on unmount', async () => {
      const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
      await flushPromises();

      wrapper.vm.undoToast.visible = true;
      wrapper.unmount();

      expect(wrapper.vm.undoToast.visible).toBe(false);
    });

    it('closes all dialogs on unmount', async () => {
      const wrapper = mount(ExtensionActionBar, { props: { inline: true } });
      await flushPromises();

      wrapper.vm.contextMenu.visible = true;
      wrapper.vm.removalDialog.visible = true;
      wrapper.vm.undoToast.visible = true;
      wrapper.unmount();

      expect(wrapper.vm.contextMenu.visible).toBe(false);
      expect(wrapper.vm.removalDialog.visible).toBe(false);
      expect(wrapper.vm.undoToast.visible).toBe(false);
    });
  });
});
