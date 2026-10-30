// Curation tool: reads scripts/asset-selection.json, downloads each model from the pinned dataset
// revision, measures it, reads its rigid-body data and rewrites packages/asset-catalog/src/catalog.json.
// Usage: node scripts/build-catalog.mjs
import { writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { readFileSync } from 'node:fs';
import {
  ASSETS_ROOT,
  CATALOG_PATH,
  download,
  glbUrl,
  measureGlb,
  readGlbPhysics,
  sha256,
} from './lib/assets.mjs';

const SELECTION = JSON.parse(
  readFileSync(resolve(import.meta.dirname, 'asset-selection.json'), 'utf8'),
);
const LICENSE = {
  id: 'CC-BY-4.0',
  attribution: 'Model from NVIDIA SimReady Assets (nvidia/simready-assets), licensed CC BY 4.0',
  url: 'https://creativecommons.org/licenses/by/4.0/',
};

const { dataset, revision } = SELECTION;
const assets = [];

for (const entry of SELECTION.assets) {
  const name = entry.source;
  const file = `models/${entry.category}/${name}.glb`;
  const buffer = await download(glbUrl(dataset, revision, name), resolve(ASSETS_ROOT, file));
  const { min, max } = measureGlb(buffer);
  const [x, y, z] = max.map((hi, i) => round(hi - min[i], 3));
  const physics = readGlbPhysics(buffer);
  assets.push({
    id: `asset.${entry.category}.${name.toLowerCase().replace(/[^a-z0-9]+/g, '_')}`,
    category: entry.category,
    name: { en: entry.en, es: entry.es },
    tags: entry.tags,
    file,
    bytes: buffer.length,
    sha256: sha256(buffer),
    dimensions: { x, y, z },
    bounds: { min: vec3(min, 4), max: vec3(max, 4) },
    ...(physics === undefined ? {} : { physics: physicsEntry(physics) }),
    source: { dataset, revision, name },
    license: LICENSE,
  });
  const mass = physics === undefined ? '' : `  ${physics.mass.toFixed(2)} kg`;
  console.log(`${name}  ${(buffer.length / 1e6).toFixed(2)} MB  ${x} × ${y} × ${z} m${mass}`);
}

await writeFile(CATALOG_PATH, `${JSON.stringify({ schemaVersion: 1, assets }, null, 2)}\n`);
console.log(`catalog.json: ${assets.length} assets (revision ${revision})`);

function physicsEntry({ mass, centerOfMass, inertia, density, materials }) {
  return {
    mass: round(mass, 3),
    centerOfMass: vec3(centerOfMass, 4),
    inertia: vec3(inertia, 4),
    ...(density === undefined ? {} : { density }),
    materials,
  };
}

function vec3([x, y, z], digits) {
  return { x: round(x, digits), y: round(y, digits), z: round(z, digits) };
}

function round(value, digits) {
  const factor = 10 ** digits;
  return Math.round(value * factor) / factor || 0;
}
