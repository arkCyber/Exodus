/**
 * Exodus Browser — TagManager component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import TagManager from './TagManager.vue';

vi.mock('@/composables/useTags', () => ({
  useTags: vi.fn(() => ({
    tags: ref([
      { id: '1', name: 'Work', color: '#ea4335' },
      { id: '2', name: 'Personal', color: '#34a853' }
    ]),
    createTag: vi.fn(),
    updateTag: vi.fn(),
    deleteTag: vi.fn()
  }))
}));

import { ref } from 'vue';

describe('TagManager', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders tag manager', () => {
    const wrapper = mount(TagManager);
    
    expect(wrapper.find('.tag-manager').exists()).toBe(true);
  });

  it('renders header with title', () => {
    const wrapper = mount(TagManager);
    
    expect(wrapper.find('.tag-manager-header h3').text()).toBe('Tags');
  });

  it('renders new tag button', () => {
    const wrapper = mount(TagManager);
    
    expect(wrapper.find('.create-tag-btn').text()).toBe('+ New Tag');
  });

  it('shows create dialog on new tag button click', async () => {
    const wrapper = mount(TagManager);
    
    await wrapper.find('.create-tag-btn').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog-overlay').exists()).toBe(true);
  });

  it('renders tag list', () => {
    const wrapper = mount(TagManager);
    
    expect(wrapper.find('.tag-list').exists()).toBe(true);
  });

  it('renders tag items', () => {
    const wrapper = mount(TagManager);
    
    const tags = wrapper.findAll('.tag-item');
    expect(tags.length).toBe(2);
  });

  it('displays tag name', () => {
    const wrapper = mount(TagManager);
    
    expect(wrapper.findAll('.tag-name')[0].text()).toBe('Work');
  });

  it('applies tag color to name', () => {
    const wrapper = mount(TagManager);
    
    const tagName = wrapper.findAll('.tag-name')[0];
    expect(tagName.attributes('style')).toContain('#ea4335');
  });

  it('applies tag color to border', () => {
    const wrapper = mount(TagManager);
    
    const tagItem = wrapper.findAll('.tag-item')[0];
    expect(tagItem.attributes('style')).toContain('border-left-color: #ea4335');
  });

  it('renders edit button on tag item', () => {
    const wrapper = mount(TagManager);
    
    const editButtons = wrapper.findAll('.tag-action-btn');
    expect(editButtons[0].text()).toBe('✎');
  });

  it('renders delete button on tag item', () => {
    const wrapper = mount(TagManager);
    
    const deleteButtons = wrapper.findAll('.tag-action-btn.delete');
    expect(deleteButtons[0].text()).toBe('×');
  });

  it('shows empty state when no tags', () => {
    const { useTags } = require('@/composables/useTags');
    useTags.mockReturnValue({
      tags: ref([]),
      createTag: vi.fn(),
      updateTag: vi.fn(),
      deleteTag: vi.fn()
    });
    
    const wrapper = mount(TagManager);
    
    expect(wrapper.find('.empty-state').text()).toBe('No tags yet. Create one to get started.');
  });

  it('opens edit dialog on edit button click', async () => {
    const wrapper = mount(TagManager);
    
    await wrapper.findAll('.tag-action-btn')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog-overlay').exists()).toBe(true);
  });

  it('shows edit dialog title when editing', async () => {
    const wrapper = mount(TagManager);
    
    await wrapper.findAll('.tag-action-btn')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog h3').text()).toBe('Edit Tag');
  });

  it('shows create dialog title when creating', async () => {
    const wrapper = mount(TagManager);
    
    await wrapper.find('.create-tag-btn').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog h3').text()).toBe('Create Tag');
  });

  it('hides dialog on overlay click', async () => {
    const wrapper = mount(TagManager);
    
    await wrapper.find('.create-tag-btn').trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.find('.dialog-overlay').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog-overlay').exists()).toBe(false);
  });

  it('does not hide dialog on dialog content click', async () => {
    const wrapper = mount(TagManager);
    
    await wrapper.find('.create-tag-btn').trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.find('.dialog').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog-overlay').exists()).toBe(true);
  });

  it('renders name input in dialog', async () => {
    const wrapper = mount(TagManager);
    
    await wrapper.find('.create-tag-btn').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('#tag-name').exists()).toBe(true);
  });

  it('renders color picker in dialog', async () => {
    const wrapper = mount(TagManager);
    
    await wrapper.find('.create-tag-btn').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.color-picker').exists()).toBe(true);
  });

  it('renders color options', async () => {
    const wrapper = mount(TagManager);
    
    await wrapper.find('.create-tag-btn').trigger('click');
    await wrapper.vm.$nextTick();
    
    const colorOptions = wrapper.findAll('.color-option');
    expect(colorOptions.length).toBe(8);
  });

  it('selects color on click', async () => {
    const wrapper = mount(TagManager);
    
    await wrapper.find('.create-tag-btn').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.color-option')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.findAll('.color-option')[0].classes()).toContain('selected');
  });

  it('renders cancel button in dialog', async () => {
    const wrapper = mount(TagManager);
    
    await wrapper.find('.create-tag-btn').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog-btn.cancel').text()).toBe('Cancel');
  });

  it('hides dialog on cancel button click', async () => {
    const wrapper = mount(TagManager);
    
    await wrapper.find('.create-tag-btn').trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.find('.dialog-btn.cancel').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog-overlay').exists()).toBe(false);
  });

  it('renders create button in dialog', async () => {
    const wrapper = mount(TagManager);
    
    await wrapper.find('.create-tag-btn').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog-btn.primary').text()).toBe('Create');
  });

  it('renders save button in edit mode', async () => {
    const wrapper = mount(TagManager);
    
    await wrapper.findAll('.tag-action-btn')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog-btn.primary').text()).toBe('Save');
  });

  it('creates tag on save', async () => {
    const { useTags } = require('@/composables/useTags');
    const createTagMock = vi.fn();
    useTags.mockReturnValue({
      tags: ref([]),
      createTag: createTagMock,
      updateTag: vi.fn(),
      deleteTag: vi.fn()
    });
    
    const wrapper = mount(TagManager);
    
    await wrapper.find('.create-tag-btn').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('#tag-name').setValue('New Tag');
    await wrapper.findAll('.color-option')[0].trigger('click');
    await wrapper.find('.dialog-btn.primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(createTagMock).toHaveBeenCalledWith('New Tag', '#ea4335');
  });

  it('does not create tag with empty name', async () => {
    const { useTags } = require('@/composables/useTags');
    const createTagMock = vi.fn();
    const alertMock = vi.fn();
    global.alert = alertMock;
    useTags.mockReturnValue({
      tags: ref([]),
      createTag: createTagMock,
      updateTag: vi.fn(),
      deleteTag: vi.fn()
    });
    
    const wrapper = mount(TagManager);
    
    await wrapper.find('.create-tag-btn').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.dialog-btn.primary').trigger('click');
    
    expect(createTagMock).not.toHaveBeenCalled();
    expect(alertMock).toHaveBeenCalledWith('Please enter a tag name');
  });

  it('updates tag on save in edit mode', async () => {
    const { useTags } = require('@/composables/useTags');
    const updateTagMock = vi.fn();
    useTags.mockReturnValue({
      tags: ref([{ id: '1', name: 'Work', color: '#ea4335' }]),
      createTag: vi.fn(),
      updateTag: updateTagMock,
      deleteTag: vi.fn()
    });
    
    const wrapper = mount(TagManager);
    
    await wrapper.findAll('.tag-action-btn')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('#tag-name').setValue('Updated Work');
    await wrapper.find('.dialog-btn.primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(updateTagMock).toHaveBeenCalledWith('1', { name: 'Updated Work', color: '#ea4335' });
  });

  it('deletes tag on delete button click', async () => {
    const { useTags } = require('@/composables/useTags');
    const deleteTagMock = vi.fn();
    const confirmMock = vi.fn(() => true);
    global.confirm = confirmMock;
    useTags.mockReturnValue({
      tags: ref([{ id: '1', name: 'Work', color: '#ea4335' }]),
      createTag: vi.fn(),
      updateTag: vi.fn(),
      deleteTag: deleteTagMock
    });
    
    const wrapper = mount(TagManager);
    
    await wrapper.find('.tag-action-btn.delete').trigger('click');
    
    expect(confirmMock).toHaveBeenCalledWith('Are you sure you want to delete this tag?');
    expect(deleteTagMock).toHaveBeenCalledWith('1');
  });

  it('does not delete tag when cancelled', async () => {
    const { useTags } = require('@/composables/useTags');
    const deleteTagMock = vi.fn();
    const confirmMock = vi.fn(() => false);
    global.confirm = confirmMock;
    useTags.mockReturnValue({
      tags: ref([{ id: '1', name: 'Work', color: '#ea4335' }]),
      createTag: vi.fn(),
      updateTag: vi.fn(),
      deleteTag: deleteTagMock
    });
    
    const wrapper = mount(TagManager);
    
    await wrapper.find('.tag-action-btn.delete').trigger('click');
    
    expect(deleteTagMock).not.toHaveBeenCalled();
  });

  it('clears form on dialog close', async () => {
    const wrapper = mount(TagManager);
    
    await wrapper.find('.create-tag-btn').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('#tag-name').setValue('Test Tag');
    await wrapper.find('.dialog-btn.cancel').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.create-tag-btn').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('#tag-name').element.value).toBe('');
  });
});
