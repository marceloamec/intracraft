// Downloads the 3D models listed in packages/asset-catalog/src/catalog.json into library-assets/.
// Files that already match the manifest (size and SHA-256) are kept. Usage: pnpm assets:fetch
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { ASSETS_ROOT, CATALOG_PATH, download, fileMatches, glbUrl, sha256 } from './lib/assets.mjs';

const { assets } = JSON.parse(readFileSync(CATALOG_PATH, 'utf8'));
let verified = 0;
const failed = [];

for (const asset of assets) {
  const path = resolve(ASSETS_ROOT, asset.file);
  if (await fileMatches(path, asset.bytes, asset.sha256)) {
    verified++;
    continue;
  }
  try {
    const { dataset, revision, name } = asset.source;
    const buffer = await download(glbUrl(dataset, revision, name), path);
    if (buffer.length !== asset.bytes || sha256(buffer) !== asset.sha256) {
      throw new Error('size or SHA-256 differs from the manifest');
    }
    verified++;
    console.log(`downloaded ${asset.file}`);
  } catch (error) {
    failed.push(asset.id);
    console.error(`FAILED ${asset.id}: ${error.message}`);
  }
}

console.log(`${verified}/${assets.length} verified`);
if (failed.length > 0) process.exit(1);
