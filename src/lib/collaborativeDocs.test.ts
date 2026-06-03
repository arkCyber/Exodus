import { describe, it, expect, beforeEach } from 'vitest';
import {
  loadCollabDocuments,
  createCollabDocument,
  updateCollabDocument,
  deleteCollabDocument,
  addCollabPeer,
  removeCollabPeer,
} from './collaborativeDocs';

describe('collaborativeDocs', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('creates and loads documents', () => {
    const doc = createCollabDocument('Test', 'hello');
    const loaded = loadCollabDocuments();
    expect(loaded.length).toBe(1);
    expect(loaded[0].id).toBe(doc.id);
    expect(loaded[0].title).toBe('Test');
  });

  it('updates document content', () => {
    const doc = createCollabDocument('A', 'v1');
    const updated = updateCollabDocument({ ...doc, content: 'v2' });
    expect(loadCollabDocuments()[0].content).toBe('v2');
    expect(updated.updated_at).toBeGreaterThanOrEqual(doc.updated_at);
  });

  it('adds and removes collaborators', () => {
    const doc = createCollabDocument('Share', '');
    const withPeer = addCollabPeer(doc, 'peer-1');
    expect(withPeer.collaborators).toContain('peer-1');
    const removed = removeCollabPeer(withPeer, 'peer-1');
    expect(removed.collaborators).not.toContain('peer-1');
  });

  it('deletes document', () => {
    const doc = createCollabDocument('Del', '');
    deleteCollabDocument(doc.id);
    expect(loadCollabDocuments().length).toBe(0);
  });
});
