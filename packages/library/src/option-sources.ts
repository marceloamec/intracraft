// Fills the options of parameters that declare `optionsFrom` (definition-schema.ts) before a built-in
// definition is validated: catalogue models (ADR-0017) or the S16 contact materials.
import { type AssetCategory, ASSET_CATEGORIES, builtInCatalog } from '@intracraft/asset-catalog';
import type { LocalizedText } from '@intracraft/core-model';
import { CONTACT_MATERIALS } from './reference-data';

interface OptionList {
  options: string[];
  labels: Record<string, LocalizedText>;
}

interface RawParameter {
  options?: string[];
  optionsFrom?: string;
  optionLabels?: Record<string, LocalizedText>;
}

/** A copy of a raw definition with every `optionsFrom` parameter's options and labels filled in. */
export function withOptionSources(source: unknown): unknown {
  if (typeof source !== 'object' || source === null || !('parameters' in source)) return source;
  const { parameters } = source as { parameters: unknown };
  if (!Array.isArray(parameters)) return source;
  return {
    ...source,
    parameters: parameters.map((raw: RawParameter) => {
      if (raw.optionsFrom === undefined) return raw;
      const extra = optionList(raw.optionsFrom);
      return {
        ...raw,
        options: [...(raw.options ?? []), ...extra.options],
        optionLabels: { ...extra.labels, ...raw.optionLabels },
      };
    }),
  };
}

function optionList(from: string): OptionList {
  if (from === 'materials') {
    return {
      options: CONTACT_MATERIALS.map((m) => m.id),
      labels: Object.fromEntries(CONTACT_MATERIALS.map((m) => [m.id, m.name])),
    };
  }
  const [kind, list] = from.split(':');
  if (kind !== 'catalogue') throw new Error(`Unknown option source "${from}"`);
  const categories = list === undefined ? [...ASSET_CATEGORIES] : list.split(',').map(category);
  const assets = categories.flatMap((c) => builtInCatalog.byCategory(c));
  return {
    options: assets.map((a) => a.id),
    labels: Object.fromEntries(assets.map((a) => [a.id, a.name])),
  };
}

function category(name: string): AssetCategory {
  const found = ASSET_CATEGORIES.find((c) => c === name.trim());
  if (found === undefined) throw new Error(`Unknown catalogue category "${name}"`);
  return found;
}
