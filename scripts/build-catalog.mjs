// Curation tool: reads scripts/asset-selection.json, downloads each model from the pinned dataset
// revision, measures it, reads its rigid-body data and collision pieces, and rewrites
// packages/asset-catalog/src/catalog.json and colliders.json.
// Usage: node scripts/build-catalog.mjs
import { writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { readFileSync } from 'node:fs';
import { readGlbColliders } from './lib/colliders.mjs';
import {
  ASSETS_ROOT,
  CATALOG_PATH,
  COLLIDERS_PATH,
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

/** SimReady material family → S16 contact material ([[S16 Contact Materials]] §2.1). */
const CONTACT_MATERIALS = {
  wood: 'wood',
  plastic: 'plastic',
  nylon: 'plastic',
  cardboard: 'cardboard',
  paper: 'cardboard',
  metal: 'steel-painted',
  glass: 'steel-painted',
  rubber: 'rubber',
};

const { dataset, revision } = SELECTION;
const assets = [];
const colliders = {};

for (const entry of SELECTION.assets) {
  const name = entry.source;
  const file = `models/${entry.category}/${name}.glb`;
  const buffer = await download(glbUrl(dataset, revision, name), resolve(ASSETS_ROOT, file));
  const { min, max } = measureGlb(buffer);
  const [x, y, z] = max.map((hi, i) => round(hi - min[i], 3));
  const physics = readGlbPhysics(buffer);
  const pieces = await readGlbColliders(buffer);
  const id = `asset.${entry.category}.${name.toLowerCase().replace(/[^a-z0-9]+/g, '_')}`;
  const contactMaterial = CONTACT_MATERIALS[pieces[0]?.family];
  if (contactMaterial === undefined)
    throw new Error(`${name}: no contact material for "${pieces[0]?.family}"`);
  colliders[id] = pieces.map((piece) => piece.points.flat().map((v) => Math.round(v * 1000)));
  assets.push({
    id,
    category: entry.category,
    name: { en: entry.en, es: entry.es },
    tags: entry.tags,
    file,
    bytes: buffer.length,
    sha256: sha256(buffer),
    dimensions: { x, y, z },
    bounds: { min: vec3(min, 4), max: vec3(max, 4) },
    ...(physics === undefined ? {} : { physics: physicsEntry(physics) }),
    contactMaterial,
    source: { dataset, revision, name },
    license: LICENSE,
  });
  const mass = physics === undefined ? '' : `  ${physics.mass.toFixed(2)} kg`;
  console.log(
    `${name}  ${(buffer.length / 1e6).toFixed(2)} MB  ${x} × ${y} × ${z} m${mass}  ${pieces.length} pieces, ${contactMaterial}`,
  );
}

await writeFile(CATALOG_PATH, `${JSON.stringify({ schemaVersion: 1, assets }, null, 2)}\n`);
await writeFile(COLLIDERS_PATH, collidersJson(colliders));
console.log(`catalog.json and colliders.json: ${assets.length} assets (revision ${revision})`);

/** One line per piece keeps the file readable in diffs. */
function collidersJson(byAsset) {
  const entries = Object.entries(byAsset).map(([id, pieces]) => {
    const lines = pieces.map((p) => `      ${JSON.stringify(p)}`).join(',\n');
    return `    ${JSON.stringify(id)}: [\n${lines}\n    ]`;
  });
  return `{\n  "schemaVersion": 1,\n  "unit": "mm",\n  "assets": {\n${entries.join(',\n')}\n  }\n}\n`;
}

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
