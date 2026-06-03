/**
 * Exodus Browser — contact directory API tests.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  contactDirectoryServiceStart,
  contactDirectoryHubInfo,
  contactGetLocalDigit,
  contactExportJson,
  contactImportJson,
  downloadContactExport,
  contactList,
  contactAdd,
  contactRemove,
  contactSearch,
  contactResolveDigitToNode,
  contactAddFriendByDigit,
  contactGetDigitForNode,
  contactGetByNode,
  contactUpdate,
  contactGetFavorites,
  contactToggleFavorite,
  contactRegisterDigitMapping,
  touchContactLastContacted,
  buildHumanContact,
} from './contactDirectory';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  isTauri: () => true,
}));

describe('contactDirectory', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('starts contact directory service', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockInfo = { storageDir: '/path', nodeId: 'node-123', inProcess: true };
    vi.mocked(invoke).mockResolvedValue(mockInfo);

    const info = await contactDirectoryServiceStart();

    expect(info).toEqual(mockInfo);
    expect(invoke).toHaveBeenCalledWith('contact_directory_service_start');
  });

  it('gets contact directory hub info', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockInfo = { storageDir: '/path', nodeId: 'node-123', inProcess: false };
    vi.mocked(invoke).mockResolvedValue(mockInfo);

    const info = await contactDirectoryHubInfo();

    expect(info).toEqual(mockInfo);
    expect(invoke).toHaveBeenCalledWith('contact_directory_hub_info');
  });

  it('gets local digit ID', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue('123456789012');

    const digit = await contactGetLocalDigit();

    expect(digit).toBe('123456789012');
    expect(invoke).toHaveBeenCalledWith('contact_get_local_digit');
  });

  it('exports contacts as JSON', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockJson = '{"version":1,"exportedAt":1234567890,"contacts":[],"groups":[]}';
    vi.mocked(invoke).mockResolvedValue(mockJson);

    const json = await contactExportJson();

    expect(json).toBe(mockJson);
    expect(invoke).toHaveBeenCalledWith('contact_export_json');
  });

  it('imports contacts from JSON', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(5);

    const count = await contactImportJson('{"contacts":[]}', true);

    expect(count).toBe(5);
    expect(invoke).toHaveBeenCalledWith('contact_import_json', { json: '{"contacts":[]}', merge: true });
  });

  it('downloads contact export', () => {
    const mockJson = '{"contacts":[]}';
    const createElementSpy = vi.spyOn(document, 'createElement').mockReturnValue({
      href: '',
      download: '',
      click: vi.fn(),
    } as any);
    const createObjectURLSpy = vi.spyOn(URL, 'createObjectURL').mockReturnValue('blob:url');
    const revokeObjectURLSpy = vi.spyOn(URL, 'revokeObjectURL');

    downloadContactExport(mockJson, 'test.json');

    expect(createObjectURLSpy).toHaveBeenCalled();
    expect(revokeObjectURLSpy).toHaveBeenCalledWith('blob:url');
    createElementSpy.mockRestore();
    createObjectURLSpy.mockRestore();
    revokeObjectURLSpy.mockRestore();
  });

  it('lists contacts', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockContacts = [
      { contact_id: '1', name: 'Test', contact_type: 'human', agent_ids: [], node_id: 'node-1', groups: [], tags: [], notes: '', is_favorite: false, is_blocked: false, created_at: 0, last_contacted: 0, contact_count: 0 },
    ];
    vi.mocked(invoke).mockResolvedValue(mockContacts);

    const contacts = await contactList();

    expect(contacts).toEqual(mockContacts);
    expect(invoke).toHaveBeenCalledWith('contact_list');
  });

  it('adds contact', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    const mockContact = { contact_id: '1', name: 'Test', contact_type: 'human', agent_ids: [], node_id: 'node-1', groups: [], tags: [], notes: '', is_favorite: false, is_blocked: false, created_at: 0, last_contacted: 0, contact_count: 0 };

    await contactAdd(mockContact);

    expect(invoke).toHaveBeenCalledWith('contact_add', { contact: mockContact });
  });

  it('removes contact', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await contactRemove('contact-123');

    expect(invoke).toHaveBeenCalledWith('contact_remove', { contactId: 'contact-123' });
  });

  it('searches contacts', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockContacts = [
      { contact_id: '1', name: 'Test', contact_type: 'human', agent_ids: [], node_id: 'node-1', groups: [], tags: [], notes: '', is_favorite: false, is_blocked: false, created_at: 0, last_contacted: 0, contact_count: 0 },
    ];
    vi.mocked(invoke).mockResolvedValue(mockContacts);

    const contacts = await contactSearch('test');

    expect(contacts).toEqual(mockContacts);
    expect(invoke).toHaveBeenCalledWith('contact_search', { query: 'test' });
  });

  it('resolves digit to node', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue('node-123');

    const nodeId = await contactResolveDigitToNode('123456789012');

    expect(nodeId).toBe('node-123');
    expect(invoke).toHaveBeenCalledWith('contact_resolve_digit_to_node', { digitId: '123456789012' });
  });

  it('adds friend by digit', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockContact = { contact_id: '1', name: 'Friend', contact_type: 'human', agent_ids: [], node_id: 'node-1', groups: [], tags: [], notes: '', is_favorite: false, is_blocked: false, created_at: 0, last_contacted: 0, contact_count: 0 };
    vi.mocked(invoke).mockResolvedValue(mockContact);

    const contact = await contactAddFriendByDigit('123456789012', 'Friend', 'user-123');

    expect(contact).toEqual(mockContact);
    expect(invoke).toHaveBeenCalledWith('contact_add_friend_by_digit', { digitId: '123456789012', name: 'Friend', userId: 'user-123' });
  });

  it('gets digit for node', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue('123456789012');

    const digit = await contactGetDigitForNode('node-123');

    expect(digit).toBe('123456789012');
    expect(invoke).toHaveBeenCalledWith('contact_get_digit_for_node', { nodeId: 'node-123' });
  });

  it('gets contact by node', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockContacts = [
      { contact_id: '1', name: 'Test', contact_type: 'human', agent_ids: [], node_id: 'node-1', groups: [], tags: [], notes: '', is_favorite: false, is_blocked: false, created_at: 0, last_contacted: 0, contact_count: 0 },
    ];
    vi.mocked(invoke).mockResolvedValue(mockContacts);

    const contacts = await contactGetByNode('node-1');

    expect(contacts).toEqual(mockContacts);
    expect(invoke).toHaveBeenCalledWith('contact_get_by_node', { nodeId: 'node-1' });
  });

  it('updates contact', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    const mockContact = { contact_id: '1', name: 'Updated', contact_type: 'human', agent_ids: [], node_id: 'node-1', groups: [], tags: [], notes: '', is_favorite: false, is_blocked: false, created_at: 0, last_contacted: 0, contact_count: 0 };

    await contactUpdate(mockContact);

    expect(invoke).toHaveBeenCalledWith('contact_update', { contact: mockContact });
  });

  it('gets favorite contacts', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockFavorites = [
      { contact_id: '1', name: 'Starred', contact_type: 'human', agent_ids: [], node_id: 'node-1', groups: [], tags: [], notes: '', is_favorite: true, is_blocked: false, created_at: 0, last_contacted: 0, contact_count: 0 },
    ];
    vi.mocked(invoke).mockResolvedValue(mockFavorites);

    const favorites = await contactGetFavorites();

    expect(favorites).toEqual(mockFavorites);
    expect(invoke).toHaveBeenCalledWith('contact_get_favorites');
  });

  it('toggles contact favorite', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(true);

    const isFavorite = await contactToggleFavorite('contact-123');

    expect(isFavorite).toBe(true);
    expect(invoke).toHaveBeenCalledWith('contact_toggle_favorite', { contactId: 'contact-123' });
  });

  it('registers digit mapping', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await contactRegisterDigitMapping('123456789012', 'node-123');

    expect(invoke).toHaveBeenCalledWith('contact_register_digit_mapping', { digitId: '123456789012', nodeId: 'node-123' });
  });

  it('touches contact last contacted', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockContact = { contact_id: '1', name: 'Test', contact_type: 'human', agent_ids: [], node_id: 'node-1', groups: [], tags: [], notes: '', is_favorite: false, is_blocked: false, created_at: 0, last_contacted: 0, contact_count: 0 };
    vi.mocked(invoke).mockResolvedValue([mockContact]);

    await touchContactLastContacted('node-1');

    expect(invoke).toHaveBeenCalledWith('contact_get_by_node', { nodeId: 'node-1' });
    expect(invoke).toHaveBeenCalledWith('contact_update', expect.objectContaining({
      contact: expect.objectContaining({
        last_contacted: expect.any(Number),
        contact_count: 1,
      }),
    }));
  });

  it('handles touch contact errors gracefully', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('Directory offline'));

    await expect(touchContactLastContacted('node-1')).resolves.not.toThrow();
  });

  it('builds human contact', () => {
    const contact = buildHumanContact({
      name: 'Test User',
      nodeId: 'node-123',
      notes: 'Test notes',
      groups: ['friends', 'family'],
    });

    expect(contact.name).toBe('Test User');
    expect(contact.node_id).toBe('node-123');
    expect(contact.contact_type).toBe('human');
    expect(contact.groups).toEqual(['friends', 'family']);
    expect(contact.notes).toBe('Test notes');
    expect(contact.is_favorite).toBe(false);
    expect(contact.is_blocked).toBe(false);
    expect(contact.contact_id).toBeDefined();
  });

  it('builds human contact with defaults', () => {
    const contact = buildHumanContact({
      name: 'Test User',
      nodeId: 'node-123',
    });

    expect(contact.groups).toEqual(['friends']);
    expect(contact.notes).toBe('');
  });

  it('handles errors gracefully', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('API error'));

    await expect(contactList()).rejects.toThrow('API error');
  });
});
