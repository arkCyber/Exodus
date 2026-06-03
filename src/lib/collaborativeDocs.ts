/**
 * Exodus Browser — local collaborative document storage (browser localStorage).
 */

export type CollabDocument = {
  id: string;
  title: string;
  content: string;
  created_at: number;
  updated_at: number;
  owner: string;
  collaborators: string[];
};

const STORAGE_KEY = 'exodus-collab-documents';

/**
 * Load all collaborative documents from localStorage.
 */
export function loadCollabDocuments(): CollabDocument[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (d): d is CollabDocument =>
        !!d &&
        typeof d === 'object' &&
        typeof (d as CollabDocument).id === 'string' &&
        typeof (d as CollabDocument).title === 'string',
    );
  } catch {
    return [];
  }
}

/**
 * Persist the full document list to localStorage.
 */
export function saveCollabDocuments(docs: CollabDocument[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(docs));
  } catch (e) {
    console.error('saveCollabDocuments failed:', e);
  }
}

/**
 * Create a new document and append to storage.
 */
export function createCollabDocument(title: string, content: string, owner = 'local-user'): CollabDocument {
  const doc: CollabDocument = {
    id: crypto.randomUUID(),
    title: title.trim() || 'Untitled',
    content,
    created_at: Math.floor(Date.now() / 1000),
    updated_at: Math.floor(Date.now() / 1000),
    owner,
    collaborators: [],
  };
  const docs = loadCollabDocuments();
  docs.push(doc);
  saveCollabDocuments(docs);
  return doc;
}

/**
 * Update document content/title in storage.
 */
export function updateCollabDocument(doc: CollabDocument): CollabDocument {
  const updated = { ...doc, updated_at: Math.floor(Date.now() / 1000) };
  const docs = loadCollabDocuments().map((d) => (d.id === updated.id ? updated : d));
  saveCollabDocuments(docs);
  return updated;
}

/**
 * Remove a document by id.
 */
export function deleteCollabDocument(id: string): void {
  saveCollabDocuments(loadCollabDocuments().filter((d) => d.id !== id));
}

/**
 * Add a collaborator peer id to a document.
 */
export function addCollabPeer(doc: CollabDocument, peer: string): CollabDocument {
  const peerTrim = peer.trim();
  if (!peerTrim || doc.collaborators.includes(peerTrim)) return doc;
  return updateCollabDocument({
    ...doc,
    collaborators: [...doc.collaborators, peerTrim],
  });
}

/**
 * Remove a collaborator from a document.
 */
export function removeCollabPeer(doc: CollabDocument, peer: string): CollabDocument {
  return updateCollabDocument({
    ...doc,
    collaborators: doc.collaborators.filter((p) => p !== peer),
  });
}
