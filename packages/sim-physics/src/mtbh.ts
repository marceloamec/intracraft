// MTBH (material to be handled): one rigid body made of a real pallet model and its load (ADR-0015).
// Body frame: X along the pallet length (travel on rollers), Y up, Z across; origin at the centre of the
// footprint, on the underside. The contact shape is the pallet construction of S10 fitted to the model.
import type { Vec3 } from '@intracraft/core-model';
import type { MassProperties, Shape } from './backend';
import { solidBoxInertia } from './mechanics';

export interface ModelBox {
  min: Vec3;
  max: Vec3;
}

/** Bounds and rigid-body data declared by a 3D model, in model coordinates (A16 `physics`). */
export interface ModelPhysics {
  bounds: ModelBox;
  mass: number;
  centerOfMass: Vec3;
  /** Principal moments about the centre of mass, model axes (kg·m²). */
  inertia: Vec3;
}

type Interval = readonly [number, number];

/** One horizontal layer of a pallet: solid intervals along its length (from the front) and across. */
export interface PalletLayer {
  fromHeight: number;
  toHeight: number;
  alongLength: readonly Interval[];
  acrossWidth: readonly Interval[];
}

export interface PalletConstruction {
  length: number;
  width: number;
  height: number;
  layers: PalletLayer[];
}

export interface MtbhSpec {
  pallet: ModelPhysics & { construction: PalletConstruction; material: string };
  load?: {
    mass: number;
    /**
     * Bounds of the loaded model (model coordinates, same origin as the pallet model). The load is
     * the block above the pallet; it is treated as uniform because the models' own mass is that of
     * empty packaging.
     */
    bounds: ModelBox;
    /** Shift of the load's centre of mass from the centre of its block (body axes, m). */
    centerOffset: Vec3;
  };
}

/** Where a model sits in the body frame: body = rotate about Y by `yaw` (quarter turns) + translation. */
export interface ModelPlacement {
  yaw: number;
  translation: Vec3;
}

export interface Mtbh {
  shapes: Shape[];
  /** Inertia products are neglected: the load offset only shifts the centre of mass (ADR-0015). */
  massProperties: MassProperties;
  material: string;
  placement: ModelPlacement;
  /** Pallet size along the body axes (m). */
  length: number;
  width: number;
  /** Total height including the load (m). */
  height: number;
}

interface OccupancyBand {
  fromHeight: number;
  toHeight: number;
  solid: readonly Interval[];
}

/** Occupancy profiles of a pallet type (S10 §3, pallet-types.json) as stacked layers. */
export function palletConstruction(type: {
  length: number;
  width: number;
  height: number;
  occupancy: { alongLength: readonly OccupancyBand[]; alongWidth: readonly OccupancyBand[] };
}): PalletConstruction {
  const layers = type.occupancy.alongLength.map((band, i) => {
    const across = type.occupancy.alongWidth[i];
    if (across?.fromHeight !== band.fromHeight || across.toHeight !== band.toHeight) {
      throw new Error(
        'Occupancy bands along the length and across the width must share their heights',
      );
    }
    return {
      fromHeight: band.fromHeight,
      toHeight: band.toHeight,
      alongLength: band.solid,
      acrossWidth: across.solid,
    };
  });
  return { length: type.length, width: type.width, height: type.height, layers };
}

export function buildMtbh({ pallet, load }: MtbhSpec): Mtbh {
  const placement = palletPlacement(pallet.bounds);
  const palletSize = sizeOf(placeBox(pallet.bounds, placement));
  const shapes: Shape[] = palletShapes(pallet.construction, palletSize);
  const parts = [
    {
      mass: pallet.mass,
      centerOfMass: placePoint(pallet.centerOfMass, placement),
      inertia: placeInertia(pallet.inertia, placement),
    },
  ];
  let height = palletSize.y;
  if (load !== undefined) {
    const block = placeBox(load.bounds, placement);
    block.min.y = palletSize.y;
    const size = sizeOf(block);
    const center = centerOf(block);
    shapes.push({ kind: 'box', halfExtents: scale(size, 0.5), offset: center });
    parts.push({
      mass: load.mass,
      centerOfMass: {
        x: center.x + load.centerOffset.x,
        y: center.y + load.centerOffset.y,
        z: center.z + load.centerOffset.z,
      },
      inertia: solidBoxInertia(load.mass, size),
    });
    height = block.max.y;
  }
  return {
    shapes,
    massProperties: combineMassProperties(parts),
    material: pallet.material,
    placement,
    length: palletSize.x,
    width: palletSize.z,
    height,
  };
}

/** Total mass, centre of mass and principal moments of rigid parts (parallel-axis theorem). */
export function combineMassProperties(parts: MassProperties[]): MassProperties {
  const mass = parts.reduce((sum, p) => sum + p.mass, 0);
  const weighted = (axis: keyof Vec3) =>
    parts.reduce((sum, p) => sum + p.mass * p.centerOfMass[axis], 0) / mass;
  const centerOfMass = { x: weighted('x'), y: weighted('y'), z: weighted('z') };
  const inertia = { x: 0, y: 0, z: 0 };
  for (const p of parts) {
    const dx = p.centerOfMass.x - centerOfMass.x;
    const dy = p.centerOfMass.y - centerOfMass.y;
    const dz = p.centerOfMass.z - centerOfMass.z;
    inertia.x += p.inertia.x + p.mass * (dy * dy + dz * dz);
    inertia.y += p.inertia.y + p.mass * (dx * dx + dz * dz);
    inertia.z += p.inertia.z + p.mass * (dx * dx + dy * dy);
  }
  return { mass, centerOfMass, inertia };
}

/** Turns the model's long side onto X and moves the footprint centre and underside to the origin. */
function palletPlacement(bounds: ModelBox): ModelPlacement {
  const longAlongZ = bounds.max.z - bounds.min.z > bounds.max.x - bounds.min.x;
  const rotated = placeBox(bounds, {
    yaw: longAlongZ ? Math.PI / 2 : 0,
    translation: { x: 0, y: 0, z: 0 },
  });
  const center = centerOf(rotated);
  return {
    yaw: longAlongZ ? Math.PI / 2 : 0,
    translation: { x: -center.x, y: -rotated.min.y, z: -center.z },
  };
}

/** One box per solid cell of each layer, scaled from the pallet type to the model size. */
function palletShapes(construction: PalletConstruction, size: Vec3): Shape[] {
  const sx = size.x / construction.length;
  const sy = size.y / construction.height;
  const sz = size.z / construction.width;
  return construction.layers.flatMap((layer) =>
    layer.alongLength.flatMap(([a0, a1]) =>
      layer.acrossWidth.map(([w0, w1]) => ({
        kind: 'box' as const,
        halfExtents: {
          x: ((a1 - a0) / 2) * sx,
          y: ((layer.toHeight - layer.fromHeight) / 2) * sy,
          z: ((w1 - w0) / 2) * sz,
        },
        offset: {
          x: size.x / 2 - ((a0 + a1) / 2) * sx,
          y: ((layer.fromHeight + layer.toHeight) / 2) * sy,
          z: ((w0 + w1) / 2) * sz - size.z / 2,
        },
      })),
    ),
  );
}

/** Exact cosine and sine of a quarter-turn yaw. */
function quarterTurn(yaw: number): [number, number] {
  const turns = (((Math.round(yaw / (Math.PI / 2)) % 4) + 4) % 4) as 0 | 1 | 2 | 3;
  return (
    [
      [1, 0],
      [0, 1],
      [-1, 0],
      [0, -1],
    ] as const
  )[turns] as [number, number];
}

function placePoint(p: Vec3, placement: ModelPlacement): Vec3 {
  const [c, s] = quarterTurn(placement.yaw);
  const t = placement.translation;
  return { x: c * p.x + s * p.z + t.x, y: p.y + t.y, z: -s * p.x + c * p.z + t.z };
}

function placeInertia(inertia: Vec3, placement: ModelPlacement): Vec3 {
  const [c] = quarterTurn(placement.yaw);
  return c === 0 ? { x: inertia.z, y: inertia.y, z: inertia.x } : { ...inertia };
}

function placeBox(box: ModelBox, placement: ModelPlacement): ModelBox {
  const a = placePoint(box.min, placement);
  const b = placePoint(box.max, placement);
  return {
    min: { x: Math.min(a.x, b.x), y: Math.min(a.y, b.y), z: Math.min(a.z, b.z) },
    max: { x: Math.max(a.x, b.x), y: Math.max(a.y, b.y), z: Math.max(a.z, b.z) },
  };
}

function sizeOf(box: ModelBox): Vec3 {
  return { x: box.max.x - box.min.x, y: box.max.y - box.min.y, z: box.max.z - box.min.z };
}

function centerOf(box: ModelBox): Vec3 {
  return {
    x: (box.min.x + box.max.x) / 2,
    y: (box.min.y + box.max.y) / 2,
    z: (box.min.z + box.max.z) / 2,
  };
}

function scale(v: Vec3, factor: number): Vec3 {
  return { x: v.x * factor, y: v.y * factor, z: v.z * factor };
}
