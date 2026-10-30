import { describe, expect, it } from 'vitest';
import type { BoxShape } from './backend';
import { solidBoxInertia } from './mechanics';
import { buildMtbh, combineMassProperties, palletConstruction } from './mtbh';

/** EUR pallet occupancy profiles from S10 §3 (pallet-types.json). */
const EUR_TYPE = {
  length: 1.2,
  width: 0.8,
  height: 0.144,
  occupancy: {
    alongLength: [
      { fromHeight: 0, toHeight: 0.022, solid: [[0, 1.2]] },
      {
        fromHeight: 0.022,
        toHeight: 0.1,
        solid: [
          [0, 0.145],
          [0.5275, 0.6725],
          [1.055, 1.2],
        ],
      },
      {
        fromHeight: 0.1,
        toHeight: 0.122,
        solid: [
          [0, 0.145],
          [0.5275, 0.6725],
          [1.055, 1.2],
        ],
      },
      { fromHeight: 0.122, toHeight: 0.144, solid: [[0, 1.2]] },
    ],
    alongWidth: [
      {
        fromHeight: 0,
        toHeight: 0.022,
        solid: [
          [0, 0.1],
          [0.3275, 0.4725],
          [0.7, 0.8],
        ],
      },
      {
        fromHeight: 0.022,
        toHeight: 0.1,
        solid: [
          [0, 0.1],
          [0.3275, 0.4725],
          [0.7, 0.8],
        ],
      },
      { fromHeight: 0.1, toHeight: 0.122, solid: [[0, 0.8]] },
      {
        fromHeight: 0.122,
        toHeight: 0.144,
        solid: [
          [0, 0.145],
          [0.18625, 0.28625],
          [0.3275, 0.4725],
          [0.51375, 0.61375],
          [0.655, 0.8],
        ],
      },
    ],
  },
} as const;

/** The SimReady wooden EUR pallet (pallet_block_a07): long side along the model's Z axis. */
const EUR_MODEL = {
  bounds: { min: { x: -0.3991, y: 0, z: -0.6 }, max: { x: 0.3991, y: 0.1408, z: 0.6 } },
  mass: 31.368,
  centerOfMass: { x: 0.0006, y: 0.0846, z: 0.001 },
  inertia: { x: 2.0337, y: 3.8501, z: 1.9923 },
};
/** The loaded EUR model (pallet_composition_a59), same origin as the empty pallet. */
const LOADED_BOUNDS = { min: { x: -0.4286, y: 0, z: -0.6 }, max: { x: 0.3991, y: 0.723, z: 0.6 } };
const WOOD_DENSITY = 700;
const ZERO = { x: 0, y: 0, z: 0 };

describe('pallet construction (S10) as contact shape', () => {
  const eur = palletConstruction(EUR_TYPE);

  it('builds the EUR pallet from 3 bottom boards, 9 blocks, 3 stringer boards and 5 deck boards', () => {
    expect(eur.layers.map((l) => l.alongLength.length * l.acrossWidth.length)).toEqual([
      3, 9, 3, 5,
    ]);
  });

  it('has a wood volume whose mass at 700 kg/m³ matches the real model (31.4 kg ±3 %)', () => {
    const mtbh = buildMtbh({ pallet: { ...EUR_MODEL, construction: eur, material: 'wood' } });
    const volume = mtbh.shapes.reduce((sum, s) => sum + 8 * boxVolume(s as BoxShape), 0);
    expect(volume * WOOD_DENSITY).toBeGreaterThan(EUR_MODEL.mass * 0.97);
    expect(volume * WOOD_DENSITY).toBeLessThan(EUR_MODEL.mass * 1.03);
  });
});

describe('MTBH body (pallet + load)', () => {
  const construction = palletConstruction(EUR_TYPE);
  const pallet = { ...EUR_MODEL, construction, material: 'wood' };

  it('turns the long side onto the body X axis and fits the shapes to the model size', () => {
    const mtbh = buildMtbh({ pallet });
    expect(mtbh.placement.yaw).toBeCloseTo(Math.PI / 2, 9);
    const extent = boundsOf(mtbh.shapes as BoxShape[]);
    expect(extent.min.x).toBeCloseTo(-0.6, 4);
    expect(extent.max.x).toBeCloseTo(0.6, 4);
    expect(extent.max.z - extent.min.z).toBeCloseTo(0.7982, 4);
    expect(extent.min.y).toBeCloseTo(0, 9);
    expect(extent.max.y).toBeCloseTo(0.1408, 4);
  });

  it('keeps the model mass and centre of mass for an empty pallet (rotated into the body frame)', () => {
    const { massProperties: m } = buildMtbh({ pallet });
    expect(m.mass).toBe(EUR_MODEL.mass);
    expect(m.centerOfMass.x).toBeCloseTo(0.001, 6);
    expect(m.centerOfMass.y).toBeCloseTo(0.0846, 6);
    expect(m.centerOfMass.z).toBeCloseTo(-0.0006, 6);
    expect(m.inertia).toEqual({
      x: EUR_MODEL.inertia.z,
      y: EUR_MODEL.inertia.y,
      z: EUR_MODEL.inertia.x,
    });
  });

  it('adds 600 kg of load as a block on the deck: 631.4 kg with the centre of mass at 415 mm', () => {
    const mtbh = buildMtbh({
      pallet,
      load: { mass: 600, bounds: LOADED_BOUNDS, centerOffset: ZERO },
    });
    const loadCenterY = (0.1408 + 0.723) / 2;
    expect(mtbh.massProperties.mass).toBeCloseTo(631.368, 6);
    expect(mtbh.massProperties.centerOfMass.y).toBeCloseTo(
      (31.368 * 0.0846 + 600 * loadCenterY) / 631.368,
      6,
    );
    expect(mtbh.shapes).toHaveLength(21);
    expect(mtbh.height).toBeCloseTo(0.723, 6);
  });

  it('moves the total centre of mass by m_load / m_total of the load offset', () => {
    const centered = buildMtbh({
      pallet,
      load: { mass: 600, bounds: LOADED_BOUNDS, centerOffset: ZERO },
    });
    const shifted = buildMtbh({
      pallet,
      load: { mass: 600, bounds: LOADED_BOUNDS, centerOffset: { x: 0.1, y: 0.05, z: -0.02 } },
    });
    const ratio = 600 / 631.368;
    const a = centered.massProperties.centerOfMass;
    const b = shifted.massProperties.centerOfMass;
    expect(b.x - a.x).toBeCloseTo(0.1 * ratio, 9);
    expect(b.y - a.y).toBeCloseTo(0.05 * ratio, 9);
    expect(b.z - a.z).toBeCloseTo(-0.02 * ratio, 9);
  });
});

describe('mass properties', () => {
  it('gives a solid box I = m·(b² + c²) / 12 about each axis', () => {
    expect(solidBoxInertia(12, { x: 1, y: 2, z: 3 })).toEqual({ x: 13, y: 10, z: 5 });
  });

  it('combines parts with the parallel-axis theorem', () => {
    const part = { mass: 1, inertia: { x: 1, y: 1, z: 1 } };
    const total = combineMassProperties([
      { ...part, centerOfMass: { x: -1, y: 0, z: 0 } },
      { ...part, centerOfMass: { x: 1, y: 0, z: 0 } },
    ]);
    expect(total.mass).toBe(2);
    expect(total.centerOfMass).toEqual({ x: 0, y: 0, z: 0 });
    expect(total.inertia).toEqual({ x: 2, y: 4, z: 4 });
  });
});

function boxVolume(shape: BoxShape): number {
  return shape.halfExtents.x * shape.halfExtents.y * shape.halfExtents.z;
}

function boundsOf(shapes: BoxShape[]) {
  const min = { x: Infinity, y: Infinity, z: Infinity };
  const max = { x: -Infinity, y: -Infinity, z: -Infinity };
  for (const { offset, halfExtents } of shapes) {
    for (const axis of ['x', 'y', 'z'] as const) {
      min[axis] = Math.min(min[axis], offset[axis] - halfExtents[axis]);
      max[axis] = Math.max(max[axis], offset[axis] + halfExtents[axis]);
    }
  }
  return { min, max };
}
