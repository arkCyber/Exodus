/**
 * Exodus Browser — Translation Client
 * Provides interface to translation service using LibreTranslate API
 */

import { invoke } from '@tauri-apps/api/core';

/** Language code */
export type LanguageCode = string;

/** Translation result */
export type TranslationResult = {
  originalText: string;
  translatedText: string;
  sourceLanguage: string;
  targetLanguage: string;
  confidence: number;
};

/** Translation settings */
export type TranslationSettings = {
  autoTranslate: boolean;
  defaultTargetLanguage: string;
  translateText: boolean;
  translateAltText: boolean;
  translateTitle: boolean;
  apiUrl?: string;
  apiKey?: string;
};

/** Translate text */
export async function translateText(text: string, targetLang: string): Promise<TranslationResult> {
  return invoke<TranslationResult>('translate_text', { text, targetLang });
}

/** Get translation settings */
export async function getTranslationSettings(): Promise<TranslationSettings> {
  return invoke<TranslationSettings>('get_translation_settings');
}

/** Update translation settings */
export async function updateTranslationSettings(settings: TranslationSettings): Promise<void> {
  return invoke<void>('update_translation_settings', { settings });
}

/** Clear translation cache */
export async function clearTranslationCache(): Promise<void> {
  return invoke<void>('clear_translation_cache');
}

/** Get supported languages */
export async function getSupportedLanguages(): Promise<[string, string, string][]> {
  return invoke<[string, string, string][]>('get_supported_languages');
}
