// Downloads the one catalogue model spike R5b draws (asset.pallets.pallet_a01, CC BY 4.0) into the folder given,
// checked against the catalogue's size and SHA-256. Usage: node spikes/r5b-viewer/fetch-pallet.mjs <folder>
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { CATALOG_PATH, download, glbUrl, sha256 } from '../../scripts/lib/assets.mjs';

const ID = 'asset.pallets.pallet_a01';
const folder = process.argv[2];
if (!folder) {
  console.error('usage: node spikes/r5b-viewer/fetch-pallet.mjs <folder>');
  process.exit(2);
}
const asset = JSON.parse(readFileSync(CATALOG_PATH, 'utf8')).assets.find((a) => a.id === ID);
const { dataset, revision, name } = asset.source;
const buffer = await download(glbUrl(dataset, revision, name), resolve(folder, 'pallet_a01.glb'));
if (buffer.length !== asset.bytes || sha256(buffer) !== asset.sha256) {
  console.error(`${ID}: size or SHA-256 differs from the catalogue`);
  process.exit(1);
}
console.log(`${ID}: ${asset.license.attribution} (${asset.license.url})`);
