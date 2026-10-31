// Convex collision pieces of a GLB model (ADR-0017). Every collider the model declares
// (KHR_physics_rigid_bodies) is split into the connected parts of its mesh, so a rack keeps the
// openings between its beams. Each part is reduced to its extreme points in fixed directions; the
// physics engine builds the convex hull of those points.
import { MeshoptDecoder } from 'meshoptimizer';
import { readGlbChunks, worldMatrices, transformPoint } from './gltf.mjs';

/** Corners kept per piece: the extreme points in this many fixed directions. */
export const MAX_POINTS = 64;
/** Pieces kept per model; those with the smallest largest-face area are dropped beyond this. */
export const MAX_PIECES = 48;
/** Vertices closer than this (m) are the same vertex when finding connected parts. */
const WELD = 0.0005;
/** Parts whose bounding-box diagonal is below this (m) are bolts and details, not colliders. */
const MIN_PIECE = 0.02;
/** Flat parts (sheets, panels) are given this thickness (m) so that their hull has a volume. */
const MIN_THICKNESS = 0.005;

const DIRECTIONS = supportDirections(MAX_POINTS);
const COMPONENT_TYPES = {
  5120: Int8Array,
  5121: Uint8Array,
  5122: Int16Array,
  5123: Uint16Array,
  5125: Uint32Array,
  5126: Float32Array,
};
const NORMALIZED_MAX = { 5120: 127, 5121: 255, 5122: 32767, 5123: 65535 };
const COMPONENTS = { SCALAR: 1, VEC2: 2, VEC3: 3, VEC4: 4 };

/**
 * Collision pieces of a model in its own frame (m, Y up), largest first:
 * `{ points: [[x, y, z], …], family, area }`, where `family` is the SimReady material family of
 * the surface ("wood", "metal", …) and `area` the largest face of the piece's bounding box (m²).
 */
export async function readGlbColliders(buffer, { limit = MAX_PIECES } = {}) {
  await MeshoptDecoder.ready;
  const { gltf, bin } = readGlbChunks(buffer);
  const world = worldMatrices(gltf);
  const views = new Map();
  const pieces = [];

  for (const node of gltf.nodes) {
    const collider = node.extensions?.KHR_physics_rigid_bodies?.collider;
    if (collider === undefined) continue;
    const geometryIndex = collider.geometry?.node ?? gltf.nodes.indexOf(node);
    const geometry = gltf.nodes[geometryIndex];
    if (geometry?.mesh === undefined) continue;
    const matrix = world.get(geometryIndex);
    for (const primitive of gltf.meshes[geometry.mesh].primitives) {
      const positions = readAccessor(gltf, bin, views, primitive.attributes.POSITION);
      const points = [];
      for (let i = 0; i < positions.length; i += 3) {
        points.push(transformPoint(matrix, [positions[i], positions[i + 1], positions[i + 2]]));
      }
      const indices =
        primitive.indices === undefined
          ? points.map((_, i) => i)
          : Array.from(readAccessor(gltf, bin, views, primitive.indices));
      const family = gltf.materials?.[primitive.material]?.name?.split('__')[1] ?? 'unknown';
      for (const part of connectedParts(points, indices)) pieces.push(toPiece(part, family));
    }
  }

  const kept = pieces.filter((p) => p.diagonal >= MIN_PIECE);
  const chosen = (kept.length > 0 ? kept : pieces).sort((a, b) => b.area - a.area);
  return chosen.slice(0, limit).map(({ points, family, area }) => ({ points, family, area }));
}

/** Groups triangle vertices into connected parts (vertices welded within WELD). */
function connectedParts(points, indices) {
  const key = ([x, y, z]) =>
    `${Math.round(x / WELD)},${Math.round(y / WELD)},${Math.round(z / WELD)}`;
  const weldId = new Map();
  const ids = points.map((p) => {
    const k = key(p);
    if (!weldId.has(k)) weldId.set(k, weldId.size);
    return weldId.get(k);
  });
  const parent = Array.from({ length: weldId.size }, (_, i) => i);
  const find = (i) => {
    while (parent[i] !== i) i = parent[i] = parent[parent[i]];
    return i;
  };
  for (let t = 0; t + 2 < indices.length; t += 3) {
    const a = find(ids[indices[t]]);
    for (const v of [indices[t + 1], indices[t + 2]]) {
      const b = find(ids[v]);
      if (a !== b) parent[b] = a;
    }
  }
  const parts = new Map();
  points.forEach((p, i) => {
    const root = find(ids[i]);
    if (!parts.has(root)) parts.set(root, []);
    parts.get(root).push(p);
  });
  return [...parts.values()];
}

/**
 * A part as a collision piece: its extreme points in the fixed directions, inflated to
 * MIN_THICKNESS where the part is flat (a hull needs volume), and its largest face area.
 */
function toPiece(points, family) {
  const { min, max } = boxOf(points);
  let corners = supportPoints(points);
  for (let axis = 0; axis < 3; axis++) {
    if (max[axis] - min[axis] >= MIN_THICKNESS) continue;
    const mid = (min[axis] + max[axis]) / 2;
    corners = corners.flatMap((p) =>
      [-1, 1].map((side) => p.map((v, i) => (i === axis ? mid + (side * MIN_THICKNESS) / 2 : v))),
    );
    corners = supportPoints(corners);
  }
  const [a, b] = max.map((v, i) => v - min[i]).sort((x, y) => y - x);
  return {
    points: corners,
    family,
    area: a * b,
    diagonal: Math.hypot(...max.map((v, i) => v - min[i])),
  };
}

/** The point furthest along each fixed direction, without repeats. */
function supportPoints(points) {
  const extremes = new Map();
  for (const d of DIRECTIONS) {
    let best = points[0];
    let bestDot = -Infinity;
    for (const p of points) {
      const dot = p[0] * d[0] + p[1] * d[1] + p[2] * d[2];
      if (dot > bestDot) {
        bestDot = dot;
        best = p;
      }
    }
    extremes.set(best.map((v) => v.toFixed(4)).join(','), best);
  }
  return [...extremes.values()];
}

function boxOf(points) {
  const min = [Infinity, Infinity, Infinity];
  const max = [-Infinity, -Infinity, -Infinity];
  for (const p of points) {
    for (let i = 0; i < 3; i++) {
      min[i] = Math.min(min[i], p[i]);
      max[i] = Math.max(max[i], p[i]);
    }
  }
  return { min, max };
}

/** The six axis directions (they keep the exact box extents) plus a Fibonacci sphere. */
function supportDirections(count) {
  const axes = [
    [1, 0, 0],
    [-1, 0, 0],
    [0, 1, 0],
    [0, -1, 0],
    [0, 0, 1],
    [0, 0, -1],
  ];
  const n = count - axes.length;
  const golden = Math.PI * (3 - Math.sqrt(5));
  const sphere = Array.from({ length: n }, (_, i) => {
    const y = 1 - (2 * (i + 0.5)) / n;
    const r = Math.sqrt(1 - y * y);
    return [Math.cos(golden * i) * r, y, Math.sin(golden * i) * r];
  });
  return [...axes, ...sphere];
}

/** Accessor values as plain numbers (normalized integers scaled to [-1, 1] or [0, 1]). */
function readAccessor(gltf, bin, views, index) {
  const accessor = gltf.accessors[index];
  const width = COMPONENTS[accessor.type];
  const Type = COMPONENT_TYPES[accessor.componentType];
  const { bytes, stride } = bufferView(gltf, bin, views, accessor.bufferView);
  const elementStride = stride || width * Type.BYTES_PER_ELEMENT;
  const scale = accessor.normalized ? 1 / NORMALIZED_MAX[accessor.componentType] : 1;
  const out = new Float64Array(accessor.count * width);
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const read = reader(view, accessor.componentType);
  for (let i = 0; i < accessor.count; i++) {
    const base = (accessor.byteOffset ?? 0) + i * elementStride;
    for (let c = 0; c < width; c++)
      out[i * width + c] = read(base + c * Type.BYTES_PER_ELEMENT) * scale;
  }
  return out;
}

function reader(view, componentType) {
  switch (componentType) {
    case 5120:
      return (o) => view.getInt8(o);
    case 5121:
      return (o) => view.getUint8(o);
    case 5122:
      return (o) => view.getInt16(o, true);
    case 5123:
      return (o) => view.getUint16(o, true);
    case 5125:
      return (o) => view.getUint32(o, true);
    default:
      return (o) => view.getFloat32(o, true);
  }
}

/** Bytes of a buffer view, decoded when it is meshopt-compressed (cached per view). */
function bufferView(gltf, bin, views, index) {
  if (views.has(index)) return views.get(index);
  const view = gltf.bufferViews[index];
  const meshopt = view.extensions?.EXT_meshopt_compression;
  let result;
  if (meshopt === undefined) {
    result = {
      bytes: bin.subarray(view.byteOffset ?? 0, (view.byteOffset ?? 0) + view.byteLength),
      stride: view.byteStride,
    };
  } else {
    const source = bin.subarray(
      meshopt.byteOffset ?? 0,
      (meshopt.byteOffset ?? 0) + meshopt.byteLength,
    );
    const bytes = new Uint8Array(meshopt.count * meshopt.byteStride);
    MeshoptDecoder.decodeGltfBuffer(
      bytes,
      meshopt.count,
      meshopt.byteStride,
      source,
      meshopt.mode,
      meshopt.filter,
    );
    result = { bytes, stride: meshopt.byteStride };
  }
  views.set(index, result);
  return result;
}
