import { createBuiltInRegistry } from '@intracraft/library';
import { describe, expect, it } from 'vitest';
import { ENGLISH_KEYS, type StringKey, TRANSLATED_KEYS, translate } from './strings';

describe('UI strings', () => {
  it('defines every key in English', () => {
    for (const key of ENGLISH_KEYS) expect(translate('en', key).length).toBeGreaterThan(0);
  });

  it('only translates keys that exist in English', () => {
    for (const keys of Object.values(TRANSLATED_KEYS)) {
      for (const key of keys) expect(ENGLISH_KEYS).toContain(key);
    }
  });

  it('has a label for every statistic of the library objects', () => {
    for (const definition of createBuiltInRegistry().list()) {
      for (const key of definition.statistics) expect(ENGLISH_KEYS, definition.id).toContain(`stat_${key}` as StringKey);
    }
  });

  it('keeps the existing Spanish and German translations', () => {
    expect(translate('es', 'language')).toBe('Idioma');
    expect(translate('de', 'language')).toBe('Sprache');
  });
});
