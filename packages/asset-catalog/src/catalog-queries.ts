// Read-only query layer over the asset manifest (the editor, CLI and future agents all read through it).
import type { Language } from '@intracraft/core-model';
import { type Asset, type AssetCategory, assetCatalogSchema } from './catalog-schema';

/** URL prefix under which the editor serves the library-assets/ folder. */
export const ASSET_URL_PREFIX = '/library-assets/';

export interface Catalog {
  list(): Asset[];
  get(id: string): Asset;
  categories(): AssetCategory[];
  byCategory(category: AssetCategory): Asset[];
  /** Case-insensitive match on name (in `language`, and English) and tags; empty text lists everything. */
  search(text: string, language?: Language): Asset[];
  urlOf(id: string): string;
}

export function createCatalog(source: unknown): Catalog {
  const assets = assetCatalogSchema.parse(source).assets;
  const byId = indexById(assets);

  const get = (id: string): Asset => {
    const asset = byId.get(id);
    if (asset === undefined) throw new Error(`Unknown asset "${id}"`);
    return asset;
  };

  return {
    list: () => [...assets],
    get,
    categories: () => [...new Set(assets.map((a) => a.category))],
    byCategory: (category) => assets.filter((a) => a.category === category),
    search: (text, language = 'en') => {
      const needle = text.trim().toLowerCase();
      if (needle === '') return [...assets];
      return assets.filter((a) => haystack(a, language).includes(needle));
    },
    urlOf: (id) => ASSET_URL_PREFIX + get(id).file,
  };
}

function indexById(assets: Asset[]): Map<string, Asset> {
  const byId = new Map<string, Asset>();
  for (const asset of assets) {
    if (byId.has(asset.id)) throw new Error(`Duplicate asset id "${asset.id}"`);
    byId.set(asset.id, asset);
  }
  return byId;
}

function haystack(asset: Asset, language: Language): string {
  return [asset.name.en, asset.name[language] ?? '', ...asset.tags].join(' ').toLowerCase();
}
