import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import CollaborativeEditing from './CollaborativeEditing.vue';

describe('CollaborativeEditing', () => {
  beforeEach(() => {
    localStorage.clear();
    vi.stubGlobal('confirm', () => true);
  });

  it('renders empty state and new document button', () => {
    const wrapper = mount(CollaborativeEditing);
    expect(wrapper.text()).toContain('Collaborative Editing');
    expect(wrapper.text()).toContain('No documents');
  });

  it('creates a document from dialog', async () => {
    const wrapper = mount(CollaborativeEditing);
    await wrapper.find('.btn-primary').trigger('click');
    const inputs = wrapper.findAll('input');
    await inputs[0].setValue('My Doc');
    await wrapper.find('.dialog .btn-primary').trigger('click');
    expect(wrapper.text()).toContain('My Doc');
  });
});
