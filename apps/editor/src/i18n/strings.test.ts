import { describe, expect, it } from 'vitest';
import { ENGLISH_KEYS, TRANSLATED_KEYS, translate } from './strings';

describe('UI strings', () => {
  it('defines every key in English', () => {
    for (const key of ENGLISH_KEYS) expect(translate('en', key).length).toBeGreaterThan(0);
  });

  it('only translates keys that exist in English', () => {
    for (const keys of Object.values(TRANSLATED_KEYS)) {
      for (const key of keys) expect(ENGLISH_KEYS).toContain(key);
    }
  });

  it('keeps the existing Spanish and German translations', () => {
    expect(translate('es', 'language')).toBe('Idioma');
    expect(translate('de', 'language')).toBe('Sprache');
  });
});
