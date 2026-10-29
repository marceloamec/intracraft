// Curation tool: reads scripts/asset-selection.json, downloads each model from the pinned dataset
// revision, measures it and rewrites packages/asset-catalog/src/catalog.json.
// Usage: node scripts/build-catalog.mjs
import { writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { readFileSync } from 'node:fs';
import { ASSETS_ROOT, CATALOG_PATH, download, glbUrl, measureGlb, sha256 } from './lib/assets.mjs';

const SELECTION = JSON.parse(
  readFileSync(resolve(import.meta.dirname, 'asset-selection.json'), 'utf8'),
);
const API = `https://huggingface.co/api/datasets/${SELECTION.dataset}`;
const LICENSE = {
  id: 'CC-BY-4.0',
  attribution: 'Model from NVIDIA SimReady Assets (nvidia/simready-assets), licensed CC BY 4.0',
  url: 'https://creativecommons.org/licenses/by/4.0/',
};

const revision = (await (await fetch(API)).json()).sha;
const assets = [];

for (const entry of SELECTION.assets) {
  const name = entry.source;
  const file = `models/${entry.category}/${name}.glb`;
  const buffer = await download(
    glbUrl(SELECTION.dataset, revision, name),
    resolve(ASSETS_ROOT, file),
  );
  const [x, y, z] = measureGlb(buffer).map((metres) => Math.round(metres * 1000) / 1000);
  assets.push({
    id: `asset.${entry.category}.${name.toLowerCase().replace(/[^a-z0-9]+/g, '_')}`,
    category: entry.category,
    name: { en: entry.en, es: entry.es },
    tags: entry.tags,
    file,
    bytes: buffer.length,
    sha256: sha256(buffer),
    dimensions: { x, y, z },
    source: { dataset: SELECTION.dataset, revision, name },
    license: LICENSE,
  });
  console.log(`${name}  ${(buffer.length / 1e6).toFixed(2)} MB  ${x} × ${y} × ${z} m`);
}

await writeFile(CATALOG_PATH, `${JSON.stringify({ schemaVersion: 1, assets }, null, 2)}\n`);
console.log(`catalog.json: ${assets.length} assets (revision ${revision})`);
