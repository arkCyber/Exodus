/**
 * Annotation & Highlighting API for Exodus Browser
 * Provides text highlighting and annotation features
 */

import { invoke } from '@tauri-apps/api/core';

export type AnnotationType = 'highlight' | 'note' | 'bookmark';
export type AnnotationColor = 'yellow' | 'green' | 'blue' | 'pink' | 'purple' | 'orange';

export interface Annotation {
  id: string;
  url: string;
  annotation_type: AnnotationType;
  color: AnnotationColor;
  text: string;
  note: string;
  position: number;
  length: number;
  created_at: number;
  updated_at: number;
}

export interface AnnotationSettings {
  enabled: boolean;
  sync_annotations: boolean;
  show_annotation_indicator: boolean;
  default_color: AnnotationColor;
}

/**
 * Create annotation
 */
export async function createAnnotation(
  url: string,
  annotationType: AnnotationType,
  color: AnnotationColor,
  text: string,
  note: string,
  position: number,
  length: number
): Promise<string> {
  return invoke('create_annotation', { 
    url, 
    annotationType, 
    color, 
    text, 
    note, 
    position, 
    length 
  });
}

/**
 * Update annotation
 */
export async function updateAnnotation(
  id: string,
  note: string,
  color: AnnotationColor
): Promise<void> {
  return invoke('update_annotation', { id, note, color });
}

/**
 * Delete annotation
 */
export async function deleteAnnotation(id: string): Promise<void> {
  return invoke('delete_annotation', { id });
}

/**
 * Get annotation by ID
 */
export async function getAnnotation(id: string): Promise<Annotation | null> {
  return invoke('get_annotation', { id });
}

/**
 * Get annotations for a URL
 */
export async function getUrlAnnotations(url: string): Promise<Annotation[]> {
  return invoke('get_url_annotations', { url });
}

/**
 * Get all annotations
 */
export async function getAllAnnotations(): Promise<Annotation[]> {
  return invoke('get_all_annotations');
}

/**
 * Search annotations
 */
export async function searchAnnotations(query: string): Promise<Annotation[]> {
  return invoke('search_annotations', { query });
}

/**
 * Enable annotations
 */
export async function enableAnnotations(): Promise<void> {
  return invoke('enable_annotations');
}

/**
 * Disable annotations
 */
export async function disableAnnotations(): Promise<void> {
  return invoke('disable_annotations');
}

/**
 * Check if annotations are enabled
 */
export async function isAnnotationsEnabled(): Promise<boolean> {
  return invoke('is_annotations_enabled');
}

/**
 * Set default annotation color
 */
export async function setAnnotationDefaultColor(color: AnnotationColor): Promise<void> {
  return invoke('set_annotation_default_color', { color });
}

/**
 * Get annotation settings
 */
export async function getAnnotationSettings(): Promise<AnnotationSettings> {
  return invoke('get_annotation_settings');
}
