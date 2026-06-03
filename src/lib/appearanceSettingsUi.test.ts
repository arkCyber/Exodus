/**
 * Exodus Browser — appearanceSettingsUi tests.
 */

import { describe, it, expect } from 'vitest';
import { appearanceSettingsStrings } from './appearanceSettingsUi';

describe('appearanceSettingsUi', () => {
  it('returns Japanese theme labels', () => {
    const s = appearanceSettingsStrings('ja');
    expect(s.sectionTitle).toContain('テーマ');
    expect(s.themeDark).toBe('ダーク');
  });

  it('returns Spanish language label', () => {
    const s = appearanceSettingsStrings('es');
    expect(s.languageLabel).toBe('Idioma');
  });
});
