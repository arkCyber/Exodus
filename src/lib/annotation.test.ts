/**
 * Exodus Browser — annotation API tests.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  createAnnotation,
  updateAnnotation,
  deleteAnnotation,
  getAnnotation,
  getUrlAnnotations,
  getAllAnnotations,
  searchAnnotations,
  enableAnnotations,
  disableAnnotations,
  isAnnotationsEnabled,
  setAnnotationDefaultColor,
  getAnnotationSettings,
} from './annotation';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('annotation', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('creates annotation', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue('annotation-123');

    const id = await createAnnotation(
      'https://example.com',
      'highlight',
      'yellow',
      'test text',
      'test note',
      10,
      20
    );

    expect(id).toBe('annotation-123');
    expect(invoke).toHaveBeenCalledWith('create_annotation', {
      url: 'https://example.com',
      annotationType: 'highlight',
      color: 'yellow',
      text: 'test text',
      note: 'test note',
      position: 10,
      length: 20,
    });
  });

  it('updates annotation', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await updateAnnotation('annotation-123', 'updated note', 'blue');

    expect(invoke).toHaveBeenCalledWith('update_annotation', {
      id: 'annotation-123',
      note: 'updated note',
      color: 'blue',
    });
  });

  it('deletes annotation', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await deleteAnnotation('annotation-123');

    expect(invoke).toHaveBeenCalledWith('delete_annotation', { id: 'annotation-123' });
  });

  it('gets annotation by ID', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockAnnotation = {
      id: 'annotation-123',
      url: 'https://example.com',
      annotation_type: 'highlight',
      color: 'yellow',
      text: 'test',
      note: 'note',
      position: 10,
      length: 20,
      created_at: Date.now(),
      updated_at: Date.now(),
    };
    vi.mocked(invoke).mockResolvedValue(mockAnnotation);

    const annotation = await getAnnotation('annotation-123');

    expect(annotation).toEqual(mockAnnotation);
    expect(invoke).toHaveBeenCalledWith('get_annotation', { id: 'annotation-123' });
  });

  it('returns null for non-existent annotation', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(null);

    const annotation = await getAnnotation('non-existent');

    expect(annotation).toBe(null);
  });

  it('gets annotations for URL', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockAnnotations = [
      { id: '1', url: 'https://example.com', annotation_type: 'highlight', color: 'yellow', text: 'test', note: '', position: 0, length: 10, created_at: Date.now(), updated_at: Date.now() },
    ];
    vi.mocked(invoke).mockResolvedValue(mockAnnotations);

    const annotations = await getUrlAnnotations('https://example.com');

    expect(annotations).toEqual(mockAnnotations);
    expect(invoke).toHaveBeenCalledWith('get_url_annotations', { url: 'https://example.com' });
  });

  it('gets all annotations', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockAnnotations = [
      { id: '1', url: 'https://example.com', annotation_type: 'highlight', color: 'yellow', text: 'test', note: '', position: 0, length: 10, created_at: Date.now(), updated_at: Date.now() },
    ];
    vi.mocked(invoke).mockResolvedValue(mockAnnotations);

    const annotations = await getAllAnnotations();

    expect(annotations).toEqual(mockAnnotations);
    expect(invoke).toHaveBeenCalledWith('get_all_annotations');
  });

  it('searches annotations', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockAnnotations = [
      { id: '1', url: 'https://example.com', annotation_type: 'highlight', color: 'yellow', text: 'test', note: '', position: 0, length: 10, created_at: Date.now(), updated_at: Date.now() },
    ];
    vi.mocked(invoke).mockResolvedValue(mockAnnotations);

    const annotations = await searchAnnotations('test');

    expect(annotations).toEqual(mockAnnotations);
    expect(invoke).toHaveBeenCalledWith('search_annotations', { query: 'test' });
  });

  it('enables annotations', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await enableAnnotations();

    expect(invoke).toHaveBeenCalledWith('enable_annotations');
  });

  it('disables annotations', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await disableAnnotations();

    expect(invoke).toHaveBeenCalledWith('disable_annotations');
  });

  it('checks if annotations are enabled', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(true);

    const enabled = await isAnnotationsEnabled();

    expect(enabled).toBe(true);
    expect(invoke).toHaveBeenCalledWith('is_annotations_enabled');
  });

  it('sets default annotation color', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await setAnnotationDefaultColor('blue');

    expect(invoke).toHaveBeenCalledWith('set_annotation_default_color', { color: 'blue' });
  });

  it('gets annotation settings', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockSettings = {
      enabled: true,
      sync_annotations: true,
      show_annotation_indicator: true,
      default_color: 'yellow',
    };
    vi.mocked(invoke).mockResolvedValue(mockSettings);

    const settings = await getAnnotationSettings();

    expect(settings).toEqual(mockSettings);
    expect(invoke).toHaveBeenCalledWith('get_annotation_settings');
  });

  it('handles errors gracefully', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('API error'));

    await expect(createAnnotation('https://example.com', 'highlight', 'yellow', 'test', '', 0, 10)).rejects.toThrow('API error');
  });
});
