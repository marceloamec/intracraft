// Shared helpers for the asset scripts: download with hash check, GLB bounding box and rigid-body data.
import { createHash } from 'node:crypto';
import { mkdir, readFile, rename, stat, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { readGltfJson, transformPoint, visitScene } from './gltf.mjs';

export const REPO_ROOT = resolve(import.meta.dirname, '../..');
export const ASSETS_ROOT = resolve(REPO_ROOT, 'library-assets');
export const CATALOG_PATH = resolve(REPO_ROOT, 'packages/asset-catalog/src/catalog.json');
export const COLLIDERS_PATH = resolve(REPO_ROOT, 'packages/asset-catalog/src/colliders.json');

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

/** Axis-aligned bounds { min, max } ([x, y, z] in metres) of the default scene of a GLB. */
export function measureGlb(buffer) {
  const gltf = readGltfJson(buffer);
  const min = [Infinity, Infinity, Infinity];
  const max = [-Infinity, -Infinity, -Infinity];
  visitScene(gltf, (node, world) => {
    if (node.mesh === undefined) return;
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
  });
  return { min, max };
}

/**
 * Rigid-body data declared with glTF `KHR_physics_rigid_bodies`: total mass, combined centre of mass
 * and principal inertia (parallel-axis sum, body axes assumed aligned with the model), density and
 * the material families named by the surfaces ("opaque__wood__new_a" → "wood").
 * Returns undefined when no node declares a mass.
 */
export function readGlbPhysics(buffer) {
  const gltf = readGltfJson(buffer);
  const bodies = [];
  visitScene(gltf, (node, world) => {
    const motion = node.extensions?.KHR_physics_rigid_bodies?.motion;
    if (motion?.mass === undefined) return;
    bodies.push({
      mass: motion.mass,
      center: transformPoint(world, motion.centerOfMass ?? [0, 0, 0]),
      inertia: motion.inertiaDiagonal ?? [0, 0, 0],
      density: node.extras?.density,
    });
  });
  if (bodies.length === 0) return undefined;

  const mass = bodies.reduce((sum, b) => sum + b.mass, 0);
  const center = [0, 1, 2].map(
    (i) => bodies.reduce((sum, b) => sum + b.mass * b.center[i], 0) / mass,
  );
  const inertia = [0, 0, 0];
  for (const b of bodies) {
    const d = b.center.map((v, i) => v - center[i]);
    inertia[0] += b.inertia[0] + b.mass * (d[1] ** 2 + d[2] ** 2);
    inertia[1] += b.inertia[1] + b.mass * (d[0] ** 2 + d[2] ** 2);
    inertia[2] += b.inertia[2] + b.mass * (d[0] ** 2 + d[1] ** 2);
  }
  const density = bodies.find((b) => b.density !== undefined)?.density;
  const families = (gltf.materials ?? []).map((m) => m.name?.split('__')[1]).filter(Boolean);
  return {
    mass,
    centerOfMass: center,
    inertia,
    ...(density === undefined ? {} : { density }),
    materials: [...new Set(families)].sort(),
  };
}

/** Quantized meshes (KHR_mesh_quantization) store normalized integers; node scale restores metres. */
const NORMALIZED_MAX = { 5120: 127, 5121: 255, 5122: 32767, 5123: 65535 };

function normalizationFactor(accessor) {
  return accessor.normalized ? 1 / NORMALIZED_MAX[accessor.componentType] : 1;
}

function corners(min, max) {
  const out = [];
  for (const x of [min[0], max[0]])
    for (const y of [min[1], max[1]]) for (const z of [min[2], max[2]]) out.push([x, y, z]);
  return out;
}
