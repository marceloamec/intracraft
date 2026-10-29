import { existsSync, statSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { ASSET_CATEGORIES, assetCatalogSchema } from './catalog-schema';
import { createCatalog } from './catalog-queries';
import rawCatalog from './catalog.json';

const ASSETS_ROOT = resolve(import.meta.dirname, '../../../library-assets');

const SAMPLE = {
  schemaVersion: 1,
  assets: [
    asset('asset.pallets.a', 'pallets', 'Wooden pallet', ['wood']),
    asset('asset.boxes.b', 'boxes', 'Cardboard box', ['cardboard']),
  ],
};

describe('catalogue manifest (catalog.json)', () => {
  const catalog = createCatalog(rawCatalog);

  it('validates against the schema', () => {
    expect(() => assetCatalogSchema.parse(rawCatalog)).not.toThrow();
  });

  it('is not empty and covers every category', () => {
    expect(catalog.list().length).toBeGreaterThan(0);
    expect(catalog.categories().sort()).toEqual([...ASSET_CATEGORIES].sort());
  });

  it('declares licence and attribution for every asset', () => {
    for (const a of catalog.list()) {
      expect(a.license.id, a.id).toBe('CC-BY-4.0');
      expect(a.license.attribution, a.id).toContain('NVIDIA');
    }
  });

  it('keeps real-world sizes (metres) between 5 cm and 30 m', () => {
    for (const a of catalog.list()) {
      for (const side of [a.dimensions.x, a.dimensions.y, a.dimensions.z]) {
        expect(side, a.id).toBeGreaterThan(0.05);
        expect(side, a.id).toBeLessThan(30);
      }
    }
  });

  it('matches the downloaded files when `pnpm assets:fetch` has been run', () => {
    for (const a of catalog.list()) {
      const path = resolve(ASSETS_ROOT, a.file);
      if (!existsSync(path)) continue;
      expect(statSync(path).size, a.id).toBe(a.bytes);
    }
  });
});

describe('catalogue queries', () => {
  const catalog = createCatalog(SAMPLE);

  it('rejects duplicate ids', () => {
    const duplicated = { ...SAMPLE, assets: [SAMPLE.assets[0], SAMPLE.assets[0]] };
    expect(() => createCatalog(duplicated)).toThrow(/duplicate/i);
  });

  it('gets an asset by id and throws for an unknown id', () => {
    expect(catalog.get('asset.boxes.b').category).toBe('boxes');
    expect(() => catalog.get('asset.nope')).toThrow(/unknown/i);
  });

  it('filters by category', () => {
    expect(catalog.byCategory('pallets').map((a) => a.id)).toEqual(['asset.pallets.a']);
  });

  it('searches name and tags in any language, ignoring case', () => {
    expect(catalog.search('PALLET').map((a) => a.id)).toEqual(['asset.pallets.a']);
    expect(catalog.search('cardboard').map((a) => a.id)).toEqual(['asset.boxes.b']);
    expect(catalog.search('palet', 'es').map((a) => a.id)).toEqual(['asset.pallets.a']);
    expect(catalog.search('')).toHaveLength(2);
  });

  it('builds a relative URL for the viewer', () => {
    expect(catalog.urlOf('asset.pallets.a')).toBe('/library-assets/models/pallets/a.glb');
  });
});

function asset(id: string, category: string, en: string, tags: string[]) {
  const file = id.split('.');
  return {
    id,
    category,
    name: { en, es: en === 'Wooden pallet' ? 'Palet de madera' : undefined },
    tags,
    file: `models/${category}/${file[2]}.glb`,
    bytes: 1000,
    sha256: 'a'.repeat(64),
    dimensions: { x: 1.2, y: 0.144, z: 0.8 },
    source: { dataset: 'codepawl/simready-glb', revision: 'abc', name: file[2] },
    license: {
      id: 'CC-BY-4.0',
      attribution: 'Model by NVIDIA SimReady, CC BY 4.0',
      url: 'https://creativecommons.org/licenses/by/4.0/',
    },
  };
}
