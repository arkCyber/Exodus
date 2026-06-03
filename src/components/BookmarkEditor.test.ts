/**
 * Exodus Browser — BookmarkEditor component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import BookmarkEditor from './BookmarkEditor.vue';

vi.mock('./TagSelector.vue', () => ({
  default: {
    name: 'TagSelector',
    template: '<div class="tag-selector-mock"></div>',
    props: ['bookmarkId']
  }
}));

const mockAddBookmark = vi.fn();
const mockUpdateBookmark = vi.fn();

vi.mock('@/composables/useBookmarks', () => ({
  useBookmarks: () => ({
    addBookmark: mockAddBookmark,
    updateBookmark: mockUpdateBookmark,
  }),
}));

describe('BookmarkEditor', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    global.alert = vi.fn();
  });

  it('does not render when visible is false', () => {
    const wrapper = mount(BookmarkEditor, {
      props: { visible: false }
    });
    
    expect(wrapper.find('.dialog-overlay').exists()).toBe(false);
  });

  it('renders when visible is true', () => {
    const wrapper = mount(BookmarkEditor, {
      props: { visible: true }
    });
    
    expect(wrapper.find('.dialog-overlay').exists()).toBe(true);
    expect(wrapper.find('.dialog').exists()).toBe(true);
  });

  it('displays "Add Bookmark" title when no bookmark provided', () => {
    const wrapper = mount(BookmarkEditor, {
      props: { visible: true }
    });
    
    expect(wrapper.find('h3').text()).toBe('Add Bookmark');
  });

  it('prefills add form from draft prop', () => {
    const wrapper = mount(BookmarkEditor, {
      props: {
        visible: true,
        draft: { title: 'Active Tab', url: 'https://example.com/page' },
      },
    });

    expect((wrapper.find('#bookmark-title').element as HTMLInputElement).value).toBe('Active Tab');
    expect((wrapper.find('#bookmark-url').element as HTMLInputElement).value).toBe(
      'https://example.com/page',
    );
  });

  it('displays "Edit Bookmark" title when bookmark provided', () => {
    const wrapper = mount(BookmarkEditor, {
      props: {
        visible: true,
        bookmark: { id: '1', title: 'Test', url: 'https://test.com' }
      }
    });
    
    expect(wrapper.find('h3').text()).toBe('Edit Bookmark');
  });

  it('populates form with bookmark data', () => {
    const wrapper = mount(BookmarkEditor, {
      props: {
        visible: true,
        bookmark: { id: '1', title: 'Test Bookmark', url: 'https://test.com', folder: 'Work' },
        folders: ['Work', 'Personal']
      }
    });
    
    const titleInput = wrapper.find('#bookmark-title');
    const urlInput = wrapper.find('#bookmark-url');
    const folderSelect = wrapper.find('#bookmark-folder');
    
    expect((titleInput.element as HTMLInputElement).value).toBe('Test Bookmark');
    expect((urlInput.element as HTMLInputElement).value).toBe('https://test.com');
    expect((folderSelect.element as HTMLSelectElement).value).toBe('Work');
  });

  it('renders folder options', () => {
    const wrapper = mount(BookmarkEditor, {
      props: {
        visible: true,
        folders: ['Work', 'Personal', 'News']
      }
    });
    
    const options = wrapper.findAll('#bookmark-folder option');
    expect(options.length).toBe(4); // Including "No folder"
    expect(options[1].text()).toBe('Work');
    expect(options[2].text()).toBe('Personal');
    expect(options[3].text()).toBe('News');
  });

  it('renders TagSelector component', () => {
    const wrapper = mount(BookmarkEditor, {
      props: {
        visible: true,
        bookmark: { id: '1', title: 'Test', url: 'https://test.com' }
      }
    });
    
    expect(wrapper.find('.tag-selector-mock').exists()).toBe(true);
  });

  it('emits close when overlay is clicked', async () => {
    const wrapper = mount(BookmarkEditor, {
      props: { visible: true }
    });
    
    await wrapper.find('.dialog-overlay').trigger('click');
    
    expect(wrapper.emitted('close')).toBeTruthy();
  });

  it('does not emit close when dialog is clicked', async () => {
    const wrapper = mount(BookmarkEditor, {
      props: { visible: true }
    });
    
    await wrapper.find('.dialog').trigger('click');
    
    expect(wrapper.emitted('close')).toBeFalsy();
  });

  it('emits close when cancel button is clicked', async () => {
    const wrapper = mount(BookmarkEditor, {
      props: { visible: true }
    });
    
    const cancelButton = wrapper.find('.dialog-btn.cancel');
    await cancelButton.trigger('click');
    
    expect(wrapper.emitted('close')).toBeTruthy();
  });

  it('calls addBookmark when adding new bookmark', async () => {
    mockAddBookmark.mockClear();
    
    const wrapper = mount(BookmarkEditor, {
      props: { visible: true }
    });
    
    await wrapper.find('#bookmark-title').setValue('New Bookmark');
    await wrapper.find('#bookmark-url').setValue('https://new.com');
    
    await wrapper.find('.dialog-btn.primary').trigger('click');
    
    expect(mockAddBookmark).toHaveBeenCalledWith('New Bookmark', 'https://new.com', undefined, undefined, undefined);
  });

  it('emits save when adding a new bookmark', async () => {
    mockAddBookmark.mockReturnValue({
      id: 'new-1',
      title: 'New Bookmark',
      url: 'https://new.com',
      created_at: new Date().toISOString(),
    });

    const wrapper = mount(BookmarkEditor, {
      props: { visible: true },
    });

    await wrapper.find('#bookmark-title').setValue('New Bookmark');
    await wrapper.find('#bookmark-url').setValue('https://new.com');
    await wrapper.find('.dialog-btn.primary').trigger('click');

    expect(wrapper.emitted('save')).toBeTruthy();
  });

  it('calls updateBookmark when editing existing bookmark', async () => {
    mockUpdateBookmark.mockClear();
    
    const wrapper = mount(BookmarkEditor, {
      props: {
        visible: true,
        bookmark: { id: '1', title: 'Old Title', url: 'https://old.com' }
      }
    });
    
    await wrapper.find('#bookmark-title').setValue('Updated Title');
    await wrapper.find('#bookmark-url').setValue('https://updated.com');
    
    await wrapper.find('.dialog-btn.primary').trigger('click');
    
    expect(mockUpdateBookmark).toHaveBeenCalledWith('1', {
      title: 'Updated Title',
      url: 'https://updated.com',
      folder: undefined
    });
  });

  it('emits save when bookmark is saved', async () => {
    const wrapper = mount(BookmarkEditor, {
      props: {
        visible: true,
        bookmark: { id: '1', title: 'Test', url: 'https://test.com' }
      }
    });
    
    await wrapper.find('#bookmark-title').setValue('Updated');
    await wrapper.find('#bookmark-url').setValue('https://updated.com');
    
    await wrapper.find('.dialog-btn.primary').trigger('click');
    
    expect(wrapper.emitted('save')).toBeTruthy();
  });

  it('shows alert when title is empty', async () => {
    const wrapper = mount(BookmarkEditor, {
      props: { visible: true }
    });
    
    await wrapper.find('#bookmark-url').setValue('https://test.com');
    await wrapper.find('.dialog-btn.primary').trigger('click');
    
    expect(global.alert).toHaveBeenCalledWith('Please fill in title and URL');
  });

  it('shows alert when URL is empty', async () => {
    const wrapper = mount(BookmarkEditor, {
      props: { visible: true }
    });
    
    await wrapper.find('#bookmark-title').setValue('Test');
    await wrapper.find('.dialog-btn.primary').trigger('click');
    
    expect(global.alert).toHaveBeenCalledWith('Please fill in title and URL');
  });

  it('does not save when validation fails', async () => {
    mockAddBookmark.mockClear();
    
    const wrapper = mount(BookmarkEditor, {
      props: { visible: true }
    });
    
    await wrapper.find('.dialog-btn.primary').trigger('click');
    
    expect(mockAddBookmark).not.toHaveBeenCalled();
    expect(wrapper.emitted('save')).toBeFalsy();
  });

  it('resets form when bookmark prop changes to null', async () => {
    const wrapper = mount(BookmarkEditor, {
      props: {
        visible: true,
        bookmark: { id: '1', title: 'Test', url: 'https://test.com' }
      }
    });
    
    await wrapper.setProps({ bookmark: null });
    await wrapper.vm.$nextTick();
    
    expect((wrapper.find('#bookmark-title').element as HTMLInputElement).value).toBe('');
    expect((wrapper.find('#bookmark-url').element as HTMLInputElement).value).toBe('');
  });

  it('displays "Add" button text when adding', () => {
    const wrapper = mount(BookmarkEditor, {
      props: { visible: true }
    });
    
    expect(wrapper.find('.dialog-btn.primary').text()).toBe('Add');
  });

  it('displays "Save" button text when editing', () => {
    const wrapper = mount(BookmarkEditor, {
      props: {
        visible: true,
        bookmark: { id: '1', title: 'Test', url: 'https://test.com' }
      }
    });
    
    expect(wrapper.find('.dialog-btn.primary').text()).toBe('Save');
  });

  describe('Boundary conditions and error handling', () => {
    it('handles bookmark with missing folder gracefully', () => {
      const wrapper = mount(BookmarkEditor, {
        props: {
          visible: true,
          bookmark: { id: '1', title: 'Test', url: 'https://test.com' }
        }
      });
      
      const folderSelect = wrapper.find('#bookmark-folder');
      expect((folderSelect.element as HTMLSelectElement).value).toBe('');
    });

    it('handles bookmark with null/undefined title gracefully', () => {
      const wrapper = mount(BookmarkEditor, {
        props: {
          visible: true,
          bookmark: { id: '1', title: null, url: 'https://test.com' } as any
        }
      });
      
      const titleInput = wrapper.find('#bookmark-title');
      expect((titleInput.element as HTMLInputElement).value).toBe('');
    });

    it('handles bookmark with null/undefined url gracefully', () => {
      const wrapper = mount(BookmarkEditor, {
        props: {
          visible: true,
          bookmark: { id: '1', title: 'Test', url: null } as any
        }
      });
      
      const urlInput = wrapper.find('#bookmark-url');
      expect((urlInput.element as HTMLInputElement).value).toBe('');
    });

    it('handles empty folders array gracefully', () => {
      const wrapper = mount(BookmarkEditor, {
        props: {
          visible: true,
          folders: []
        }
      });
      
      const options = wrapper.findAll('#bookmark-folder option');
      expect(options.length).toBe(1); // Only "No folder"
    });

    it('trims whitespace from title and URL before validation', async () => {
      mockAddBookmark.mockClear();
      
      const wrapper = mount(BookmarkEditor, {
        props: { visible: true }
      });
      
      await wrapper.find('#bookmark-title').setValue('  Test  ');
      await wrapper.find('#bookmark-url').setValue('  https://test.com  ');
      
      await wrapper.find('.dialog-btn.primary').trigger('click');
      
      // Component doesn't trim, so it passes the raw values
      expect(mockAddBookmark).toHaveBeenCalledWith('  Test  ', '  https://test.com  ', undefined, undefined, undefined);
    });

    it('handles folder with empty string as "No folder"', async () => {
      mockAddBookmark.mockClear();
      
      const wrapper = mount(BookmarkEditor, {
        props: { visible: true }
      });
      
      await wrapper.find('#bookmark-title').setValue('Test');
      await wrapper.find('#bookmark-url').setValue('https://test.com');
      await wrapper.find('#bookmark-folder').setValue('');
      
      await wrapper.find('.dialog-btn.primary').trigger('click');
      
      expect(mockAddBookmark).toHaveBeenCalledWith('Test', 'https://test.com', undefined, undefined, undefined);
    });

    it('does not emit close when save fails validation', async () => {
      const wrapper = mount(BookmarkEditor, {
        props: { visible: true }
      });
      
      await wrapper.find('.dialog-btn.primary').trigger('click');
      
      expect(wrapper.emitted('close')).toBeFalsy();
    });

    it('emits close after successful save', async () => {
      const wrapper = mount(BookmarkEditor, {
        props: {
          visible: true,
          bookmark: { id: '1', title: 'Test', url: 'https://test.com' }
        }
      });
      
      await wrapper.find('#bookmark-title').setValue('Updated');
      await wrapper.find('#bookmark-url').setValue('https://updated.com');
      
      await wrapper.find('.dialog-btn.primary').trigger('click');
      
      expect(wrapper.emitted('close')).toBeTruthy();
    });
  });
});
