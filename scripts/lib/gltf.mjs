// Minimal glTF binary (GLB) reading shared by the asset scripts: chunks, node matrices, points.

const IDENTITY = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];

export function readGltfJson(buffer) {
  return readGlbChunks(buffer).gltf;
}

/** The JSON chunk (parsed) and the binary chunk of a GLB. */
export function readGlbChunks(buffer) {
  if (buffer.readUInt32LE(0) !== 0x46546c67) throw new Error('Not a GLB file');
  const jsonLength = buffer.readUInt32LE(12);
  const gltf = JSON.parse(buffer.toString('utf8', 20, 20 + jsonLength));
  const binStart = 20 + jsonLength;
  const bin =
    binStart + 8 <= buffer.length
      ? new Uint8Array(
          buffer.buffer,
          buffer.byteOffset + binStart + 8,
          buffer.readUInt32LE(binStart),
        )
      : new Uint8Array(0);
  return { gltf, bin };
}

/** Calls `visit(node, worldMatrix, index)` for every node of the default scene. */
export function visitScene(gltf, visit) {
  const walk = (index, parent) => {
    const node = gltf.nodes[index];
    const world = multiply(parent, localMatrix(node));
    visit(node, world, index);
    for (const child of node.children ?? []) walk(child, world);
  };
  for (const root of gltf.scenes?.[gltf.scene ?? 0]?.nodes ?? []) walk(root, IDENTITY);
}

/** World matrix of every node of the default scene, by node index. */
export function worldMatrices(gltf) {
  const matrices = new Map();
  visitScene(gltf, (_node, world, index) => matrices.set(index, world));
  return matrices;
}

export function transformPoint(m, [x, y, z]) {
  return [
    m[0] * x + m[4] * y + m[8] * z + m[12],
    m[1] * x + m[5] * y + m[9] * z + m[13],
    m[2] * x + m[6] * y + m[10] * z + m[14],
  ];
}

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
