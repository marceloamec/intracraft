// Shared helpers for the asset scripts: download with hash check and GLB bounding-box measurement.
import { createHash } from 'node:crypto';
import { mkdir, readFile, rename, stat, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';

export const REPO_ROOT = resolve(import.meta.dirname, '../..');
export const ASSETS_ROOT = resolve(REPO_ROOT, 'library-assets');
export const CATALOG_PATH = resolve(REPO_ROOT, 'packages/asset-catalog/src/catalog.json');

export function glbUrl(dataset, revision, name) {
  return `https://huggingface.co/datasets/${dataset}/resolve/${revision}/glb/${encodeURIComponent(name)}.glb`;
}

export function sha256(buffer) {
  return createHash('sha256').update(buffer).digest('hex');
}

/** Downloads `url` to `destination` (via a .part file) and returns the bytes. */
export async function download(url, destination) {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`HTTP ${response.status} for ${url}`);
  const buffer = Buffer.from(await response.arrayBuffer());
  await mkdir(dirname(destination), { recursive: true });
  await writeFile(`${destination}.part`, buffer);
  await rename(`${destination}.part`, destination);
  return buffer;
}

export async function fileMatches(path, bytes, hash) {
  try {
    if ((await stat(path)).size !== bytes) return false;
    return sha256(await readFile(path)) === hash;
  } catch {
    return false;
  }
}

/** Axis-aligned size [x, y, z] in metres of the default scene of a GLB (node transforms applied). */
export function measureGlb(buffer) {
  if (buffer.readUInt32LE(0) !== 0x46546c67) throw new Error('Not a GLB file');
  const jsonLength = buffer.readUInt32LE(12);
  const gltf = JSON.parse(buffer.toString('utf8', 20, 20 + jsonLength));
  const min = [Infinity, Infinity, Infinity];
  const max = [-Infinity, -Infinity, -Infinity];
  const roots = gltf.scenes?.[gltf.scene ?? 0]?.nodes ?? [];

  const visit = (index, parent) => {
    const node = gltf.nodes[index];
    const world = multiply(parent, localMatrix(node));
    if (node.mesh !== undefined) {
      for (const primitive of gltf.meshes[node.mesh].primitives) {
        const accessor = gltf.accessors[primitive.attributes.POSITION];
        const scale = normalizationFactor(accessor);
        const lo = accessor.min.map((v) => v * scale);
        const hi = accessor.max.map((v) => v * scale);
        for (const corner of corners(lo, hi)) {
          const p = transformPoint(world, corner);
          for (let i = 0; i < 3; i++) {
            min[i] = Math.min(min[i], p[i]);
            max[i] = Math.max(max[i], p[i]);
          }
        }
      }
    }
    for (const child of node.children ?? []) visit(child, world);
  };
  for (const root of roots) visit(root, IDENTITY);
  return max.map((value, i) => value - min[i]);
}

/** Quantized meshes (KHR_mesh_quantization) store normalized integers; node scale restores metres. */
const NORMALIZED_MAX = { 5120: 127, 5121: 255, 5122: 32767, 5123: 65535 };

function normalizationFactor(accessor) {
  return accessor.normalized ? 1 / NORMALIZED_MAX[accessor.componentType] : 1;
}

const IDENTITY = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];

function localMatrix(node) {
  if (node.matrix) return node.matrix;
  const [qx, qy, qz, qw] = node.rotation ?? [0, 0, 0, 1];
  const [sx, sy, sz] = node.scale ?? [1, 1, 1];
  const [tx, ty, tz] = node.translation ?? [0, 0, 0];
  return [
    (1 - 2 * (qy * qy + qz * qz)) * sx,
    2 * (qx * qy + qz * qw) * sx,
    2 * (qx * qz - qy * qw) * sx,
    0,
    2 * (qx * qy - qz * qw) * sy,
    (1 - 2 * (qx * qx + qz * qz)) * sy,
    2 * (qy * qz + qx * qw) * sy,
    0,
    2 * (qx * qz + qy * qw) * sz,
    2 * (qy * qz - qx * qw) * sz,
    (1 - 2 * (qx * qx + qy * qy)) * sz,
    0,
    tx,
    ty,
    tz,
    1,
  ];
}

/** Column-major 4x4 product a·b. */
function multiply(a, b) {
  const out = new Array(16).fill(0);
  for (let col = 0; col < 4; col++)
    for (let row = 0; row < 4; row++)
      for (let k = 0; k < 4; k++) out[col * 4 + row] += a[k * 4 + row] * b[col * 4 + k];
  return out;
}

function corners(min, max) {
  const out = [];
  for (const x of [min[0], max[0]])
    for (const y of [min[1], max[1]]) for (const z of [min[2], max[2]]) out.push([x, y, z]);
  return out;
}

function transformPoint(m, [x, y, z]) {
  return [
    m[0] * x + m[4] * y + m[8] * z + m[12],
    m[1] * x + m[5] * y + m[9] * z + m[13],
    m[2] * x + m[6] * y + m[10] * z + m[14],
  ];
}
