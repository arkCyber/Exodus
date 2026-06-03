/**
 * Exodus Browser — CollaborativeEditing component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import CollaborativeEditing from './CollaborativeEditing.vue';

vi.mock('$lib/confirm', () => ({
  CONFIRM_DIALOG_KEY: 'confirm-dialog'
}));

vi.mock('$lib/collaborativeDocs', () => ({
  loadCollabDocuments: vi.fn(() => []),
  createCollabDocument: vi.fn((title, content) => ({
    id: 'doc-1',
    title,
    content,
    updated_at: Date.now(),
    collaborators: []
  })),
  updateCollabDocument: vi.fn((doc) => doc),
  deleteCollabDocument: vi.fn(),
  addCollabPeer: vi.fn((doc, peer) => ({
    ...doc,
    collaborators: [...doc.collaborators, peer]
  })),
  removeCollabPeer: vi.fn((doc, peer) => ({
    ...doc,
    collaborators: doc.collaborators.filter(c => c !== peer)
  }))
}));

describe('CollaborativeEditing', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders collaborative editing component', () => {
    const wrapper = mount(CollaborativeEditing);
    
    expect(wrapper.find('.collaborative-editing').exists()).toBe(true);
  });

  it('renders header with title', () => {
    const wrapper = mount(CollaborativeEditing);
    
    expect(wrapper.find('.header h2').text()).toBe('Collaborative Editing');
  });

  it('renders new document button', () => {
    const wrapper = mount(CollaborativeEditing);
    
    expect(wrapper.find('.header .btn-primary').text()).toBe('New Document');
  });

  it('shows new document dialog on button click', async () => {
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.header .btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog-overlay').exists()).toBe(true);
  });

  it('hides new document dialog on overlay click', async () => {
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.header .btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.find('.dialog-overlay').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog-overlay').exists()).toBe(false);
  });

  it('has correct ARIA role on dialog', async () => {
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.header .btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog').attributes('role')).toBe('dialog');
  });

  it('renders new document dialog title', async () => {
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.header .btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog h3').text()).toBe('New Document');
  });

  it('renders title input in new document dialog', async () => {
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.header .btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    const labels = wrapper.findAll('.dialog label');
    expect(labels[0].text()).toBe('Title');
  });

  it('renders content textarea in new document dialog', async () => {
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.header .btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    const labels = wrapper.findAll('.dialog label');
    expect(labels[1].text()).toBe('Initial content');
  });

  it('renders cancel button in new document dialog', async () => {
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.header .btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.dialog .btn-secondary');
    expect(buttons[0].text()).toBe('Cancel');
  });

  it('hides dialog on cancel button click', async () => {
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.header .btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.findAll('.dialog .btn-secondary')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog-overlay').exists()).toBe(false);
  });

  it('renders create button in new document dialog', async () => {
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.header .btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog .btn-primary').text()).toBe('Create');
  });

  it('creates document with title and content', async () => {
    const { createCollabDocument, loadCollabDocuments } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([{ id: 'doc-1', title: 'Test', content: 'Content', updated_at: Date.now(), collaborators: [] }]);
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.header .btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.dialog input').setValue('Test Document');
    await wrapper.find('.dialog textarea').setValue('Test content');
    await wrapper.find('.dialog .btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(createCollabDocument).toHaveBeenCalledWith('Test Document', 'Test content');
  });

  it('does not create document with empty title', async () => {
    const { createCollabDocument } = require('$lib/collaborativeDocs');
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.header .btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.dialog .btn-primary').trigger('click');
    
    expect(createCollabDocument).not.toHaveBeenCalled();
  });

  it('emits status on document creation', async () => {
    const { createCollabDocument, loadCollabDocuments } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([{ id: 'doc-1', title: 'Test', content: 'Content', updated_at: Date.now(), collaborators: [] }]);
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.header .btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.dialog input').setValue('Test Document');
    await wrapper.find('.dialog .btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Created "Test Document"']);
  });

  it('renders editor container', () => {
    const wrapper = mount(CollaborativeEditing);
    
    expect(wrapper.find('.editor-container').exists()).toBe(true);
  });

  it('renders sidebar', () => {
    const wrapper = mount(CollaborativeEditing);
    
    expect(wrapper.find('.sidebar').exists()).toBe(true);
  });

  it('renders documents header', () => {
    const wrapper = mount(CollaborativeEditing);
    
    expect(wrapper.find('.sidebar h3').text()).toBe('Documents');
  });

  it('shows empty state when no documents', () => {
    const wrapper = mount(CollaborativeEditing);
    
    expect(wrapper.find('.empty-state').exists()).toBe(true);
    expect(wrapper.find('.empty-state').text()).toBe('No documents');
  });

  it('renders document items', () => {
    const { loadCollabDocuments } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: [] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    const documents = wrapper.findAll('.document-item');
    expect(documents.length).toBe(1);
  });

  it('displays document title', () => {
    const { loadCollabDocuments } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: [] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    expect(wrapper.find('.doc-title').text()).toBe('Test Doc');
  });

  it('displays formatted update date', () => {
    const { loadCollabDocuments } = require('$lib/collaborativeDocs');
    const timestamp = Date.now() / 1000;
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: timestamp, collaborators: [] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    expect(wrapper.find('.doc-meta').text()).toContain('Updated:');
  });

  it('displays collaborator count', () => {
    const { loadCollabDocuments } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: ['peer-1', 'peer-2'] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    expect(wrapper.find('.doc-meta').text()).toContain('2 collaborator(s)');
  });

  it('applies active class to selected document', async () => {
    const { loadCollabDocuments } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: [] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.document-item').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.document-item').classes()).toContain('active');
  });

  it('renders delete button on document item', () => {
    const { loadCollabDocuments } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: [] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    expect(wrapper.find('.btn-icon.delete').exists()).toBe(true);
  });

  it('shows editor main when document is selected', async () => {
    const { loadCollabDocuments } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: [] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.document-item').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.editor-main').exists()).toBe(true);
  });

  it('shows placeholder when no document selected', () => {
    const wrapper = mount(CollaborativeEditing);
    
    expect(wrapper.find('.editor-placeholder').exists()).toBe(true);
    expect(wrapper.find('.editor-placeholder').text()).toBe('Select or create a document');
  });

  it('renders editor toolbar', async () => {
    const { loadCollabDocuments } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: [] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.document-item').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.editor-toolbar').exists()).toBe(true);
  });

  it('renders title input in editor', async () => {
    const { loadCollabDocuments } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: [] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.document-item').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.title-input').exists()).toBe(true);
  });

  it('renders share button in editor', async () => {
    const { loadCollabDocuments } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: [] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.document-item').trigger('click');
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.editor-toolbar .btn-secondary');
    expect(buttons[0].text()).toBe('Share');
  });

  it('renders save button in editor', async () => {
    const { loadCollabDocuments } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: [] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.document-item').trigger('click');
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.editor-toolbar .btn-secondary');
    expect(buttons[1].text()).toBe('Save');
  });

  it('renders editor textarea', async () => {
    const { loadCollabDocuments } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: [] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.document-item').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.editor-textarea').exists()).toBe(true);
  });

  it('displays document content in textarea', async () => {
    const { loadCollabDocuments } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Test content', updated_at: Date.now(), collaborators: [] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.document-item').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.editor-textarea').element.value).toBe('Test content');
  });

  it('shows share dialog on share button click', async () => {
    const { loadCollabDocuments } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: [] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.document-item').trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.findAll('.editor-toolbar .btn-secondary')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.findAll('.dialog-overlay')[1].exists()).toBe(true);
  });

  it('renders share dialog title', async () => {
    const { loadCollabDocuments } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: [] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.document-item').trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.findAll('.editor-toolbar .btn-secondary')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.findAll('.dialog h3')[1].text()).toBe('Share document');
  });

  it('renders peer ID input in share dialog', async () => {
    const { loadCollabDocuments } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: [] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.document-item').trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.findAll('.editor-toolbar .btn-secondary')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    const labels = wrapper.findAll('.dialog label');
    expect(labels[2].text()).toBe('Peer ID');
  });

  it('shares document with peer', async () => {
    const { loadCollabDocuments, addCollabPeer } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: [] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.document-item').trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.findAll('.editor-toolbar .btn-secondary')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.dialog input')[1].setValue('peer-123');
    await wrapper.findAll('.dialog .btn-primary')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(addCollabPeer).toHaveBeenCalled();
  });

  it('emits status on share', async () => {
    const { loadCollabDocuments, addCollabPeer } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: [] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.document-item').trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.findAll('.editor-toolbar .btn-secondary')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.dialog input')[1].setValue('peer-123');
    await wrapper.findAll('.dialog .btn-primary')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Collaborator added']);
  });

  it('renders collaborators list', async () => {
    const { loadCollabDocuments } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: ['peer-1', 'peer-2'] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.document-item').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.collaborators').exists()).toBe(true);
  });

  it('displays collaborator chips', async () => {
    const { loadCollabDocuments } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: ['peer-1', 'peer-2'] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.document-item').trigger('click');
    await wrapper.vm.$nextTick();
    
    const chips = wrapper.findAll('.collab-chip');
    expect(chips.length).toBe(2);
  });

  it('removes collaborator on remove button click', async () => {
    const { loadCollabDocuments, removeCollabPeer } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: ['peer-1'] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.document-item').trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.find('.collab-chip .btn-icon').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(removeCollabPeer).toHaveBeenCalled();
  });

  it 'renders remote cursors hint', async () => {
    const { loadCollabDocuments } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: [] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.document-item').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.cursors-hint').exists()).toBe(true);
  });

  it('displays cursor chips with user info', async () => {
    const { loadCollabDocuments } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: [] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.document-item').trigger('click');
    await wrapper.vm.$nextTick();
    
    const cursorChips = wrapper.findAll('.cursor-chip');
    expect(cursorChips.length).toBe(2);
  });

  it('saves document on save button click', async () => {
    const { loadCollabDocuments, updateCollabDocument } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: [] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.document-item').trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.findAll('.editor-toolbar .btn-secondary')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(updateCollabDocument).toHaveBeenCalled();
  });

  it('emits status on save', async () => {
    const { loadCollabDocuments, updateCollabDocument } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: [] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.document-item').trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.findAll('.editor-toolbar .btn-secondary')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Document saved']);
  });

  it('loads documents on mount', () => {
    const { loadCollabDocuments } = require('$lib/collaborativeDocs');
    
    mount(CollaborativeEditing);
    
    expect(loadCollabDocuments).toHaveBeenCalled();
  });

  it('deletes document after confirmation', async () => {
    const { loadCollabDocuments, deleteCollabDocument } = require('$lib/collaborativeDocs');
    loadCollabDocuments.mockReturnValue([
      { id: 'doc-1', title: 'Test Doc', content: 'Content', updated_at: Date.now(), collaborators: [] }
    ]);
    
    const wrapper = mount(CollaborativeEditing);
    
    await wrapper.find('.btn-icon.delete').trigger('click');
    await wrapper.vm.$nextTick();
    
    // Since confirmDialog is not injected, it should use window.confirm
    // We'll mock window.confirm
    const originalConfirm = window.confirm;
    window.confirm = vi.fn(() => true);
    
    await wrapper.find('.btn-icon.delete').trigger('click');
    await wrapper.vm.$nextTick();
    
    window.confirm = originalConfirm;
    
    expect(deleteCollabDocument).toHaveBeenCalled();
  });
});
