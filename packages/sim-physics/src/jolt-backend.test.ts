import { GRAVITY, type Vec3 } from '@intracraft/core-model';
import { contactPair } from '@intracraft/library';
import { describe, expect, it } from 'vitest';
import type { BodyDesc, BodyHandle, ContactPairLookup, PhysicsBackend } from './backend';
import { createJoltBackend } from './jolt-backend';
import { rollerAngularVelocity } from './mechanics';

const DT = 1 / 120;
const BOX_MASS = 1000;
const BOX_HALF = { x: 0.6, y: 0.072, z: 0.4 };
const ZERO = { x: 0, y: 0, z: 0 };

describe('Jolt backend: contacts', () => {
  it('rests a 1000 kg box on the floor within 2 mm of the surface (slop lowered to 1 mm)', async () => {
    await withWorld(pairs({ 'concrete|wood': [0.5, 0] }), (world) => {
      world.addBody(floor());
      const box = world.addBody(box1000({ x: 0, y: BOX_HALF.y + 0.002, z: 0 }));
      run(world, 1);
      expect(world.getPose(box).position.y).toBeCloseTo(BOX_HALF.y, 2);
    });
  });

  it('decelerates a sliding box at μ·g with the pair friction (μ = 0.5 → stops in 0.102 m)', async () => {
    await withWorld(pairs({ 'concrete|wood': [0.5, 0] }), (world) => {
      world.addBody(floor());
      const box = world.addBody(box1000({ x: 0, y: BOX_HALF.y, z: 0 }));
      world.setVelocity(box, { x: 1, y: 0, z: 0 }, ZERO);
      run(world, 1);
      expect(world.getPose(box).position.x).toBeCloseTo(1 / (2 * 0.5 * GRAVITY), 2);
    });
  });

  it('brakes a cardboard box on steel with the S16 pair (μ = 0.3 → stops in 0.170 m)', async () => {
    await withWorld(contactPair, (world) => {
      world.addBody({ ...floor(), material: 'steel-zinc' });
      const box = world.addBody({ ...box1000({ x: 0, y: BOX_HALF.y, z: 0 }), material: 'cardboard' });
      world.setVelocity(box, { x: 1, y: 0, z: 0 }, ZERO);
      run(world, 1);
      expect(world.getPose(box).position.x).toBeCloseTo(1 / (2 * 0.3 * GRAVITY), 2);
    });
  });

  it('lets a per-instance friction replace the pair value (μ = 0.2 → 0.255 m)', async () => {
    await withWorld(pairs({ 'concrete|wood': [0.5, 0] }), (world) => {
      world.addBody({ ...floor(), friction: 0.2 });
      const box = world.addBody(box1000({ x: 0, y: BOX_HALF.y, z: 0 }));
      world.setVelocity(box, { x: 1, y: 0, z: 0 }, ZERO);
      run(world, 1);
      expect(world.getPose(box).position.x).toBeCloseTo(1 / (2 * 0.2 * GRAVITY), 2);
    });
  });

  it('bounces with the pair restitution even at 0.25 m/s (below the engine default threshold)', async () => {
    for (const restitution of [0, 0.5]) {
      await withWorld(pairs({ 'concrete|wood': [0.5, restitution] }), (world) => {
        world.addBody(floor());
        const drop = 0.25 ** 2 / (2 * GRAVITY);
        const box = world.addBody(box1000({ x: 0, y: BOX_HALF.y + drop, z: 0 }));
        let rebound = 0;
        for (let i = 0; i < 60; i++) {
          world.step(DT);
          rebound = Math.max(rebound, world.getVelocity(box).linear.y);
        }
        if (restitution === 0) expect(rebound).toBeLessThan(0.01);
        else expect(rebound).toBeCloseTo(restitution * 0.25, 1);
      });
    }
  });
});

describe('Jolt backend: impacts', () => {
  it('reports approach speed and energy ½·m·v² of a box hitting a wall at 0.25 m/s (31.25 J)', async () => {
    await withWorld(
      pairs({ 'concrete|wood': [0, 0], 'steel-painted|wood': [0.3, 0.1] }),
      (world) => {
        world.addBody(floor());
        const wall = world.addBody(wallAt(0.5));
        const box = world.addBody(box1000({ x: -BOX_HALF.x, y: BOX_HALF.y, z: 0 }));
        world.setVelocity(box, { x: 0.25, y: 0, z: 0 }, ZERO);
        run(world, 3);
        const hits = world.takeImpacts();
        // Only the wall: resting on the floor must not count as an impact.
        expect(hits.map((i) => i.bodies)).toEqual([[wall, box]]);
        expect(hits[0]?.speed).toBeCloseTo(0.25, 2);
        expect(hits[0]?.energy).toBeGreaterThan(31.25 * 0.97);
        expect(hits[0]?.energy).toBeLessThan(31.25 * 1.03);
        expect(hits[0]?.time).toBeCloseTo(2, 1);
        expect(world.takeImpacts()).toEqual([]);
      },
    );
  });
});

describe('Jolt backend: driven surfaces', () => {
  it('carries a box at 0.25 m/s on kinematic Ø89 rollers spinning at ω = v / r', async () => {
    await withWorld(pairs({ 'steel-zinc|wood': [0.35, 0] }), (world) => {
      const rollers = rollerRow(world, 10, 0.2063, 0.089);
      const box = world.addBody(box1000({ x: 0.9, y: 0.0445 + BOX_HALF.y, z: 0 }));
      for (const roller of rollers) {
        world.setVelocity(roller, ZERO, { x: 0, y: 0, z: -rollerAngularVelocity(0.25, 0.089) });
      }
      run(world, 1);
      expect(world.getVelocity(box).linear.x).toBeCloseTo(0.25, 3);
    });
  });

  it('carries a box on a static strip with a surface velocity (chain plan B)', async () => {
    await withWorld(pairs({ 'concrete|wood': [0.5, 0] }), (world) => {
      const strip = world.addBody(floor());
      const box = world.addBody(box1000({ x: 0, y: BOX_HALF.y, z: 0 }));
      expect(world.setSurfaceVelocity(strip, { x: 0.25, y: 0, z: 0 })).toBe(true);
      run(world, 1);
      expect(world.getVelocity(box).linear.x).toBeCloseTo(0.25, 2);
    });
  });
});

describe('Jolt backend: mass properties', () => {
  it('tips a half-overhanging box only when its centre of mass is past the edge', async () => {
    for (const [offset, tips] of [
      [0.1, true],
      [-0.1, false],
    ] as const) {
      await withWorld(pairs({ 'concrete|wood': [0.5, 0] }), (world) => {
        world.addBody({
          ...floor(),
          pose: pose({ x: -2, y: -0.05, z: 0 }),
          shapes: [box({ x: 2, y: 0.05, z: 2 })],
        });
        const block = world.addBody({
          ...box1000({ x: 0, y: 0.1, z: 0 }),
          shapes: [box({ x: 0.5, y: 0.1, z: 0.3 })],
          massProperties: {
            mass: 100,
            centerOfMass: { x: offset, y: 0, z: 0 },
            inertia: { x: 4, y: 10, z: 9 },
          },
        });
        run(world, 1);
        const tilt = 2 * Math.acos(Math.min(1, Math.abs(world.getPose(block).rotation.w)));
        if (tips) expect(tilt).toBeGreaterThan(0.17);
        else expect(tilt).toBeLessThan(0.01);
      });
    }
  });

  it('rejects a dynamic body without mass properties', async () => {
    await withWorld(pairs({}), (world) => {
      const withoutMass = box1000(ZERO);
      delete withoutMass.massProperties;
      expect(() => world.addBody(withoutMass)).toThrow(/mass/);
    });
  });
});

describe('Jolt backend: convex hulls and sleeping (ADR-0017)', () => {
  it('rests a box given as a convex hull of its 8 corners like the native box (±1 mm)', async () => {
    await withWorld(pairs({ 'concrete|wood': [0.5, 0] }), (world) => {
      world.addBody(floor());
      const native = world.addBody(box1000({ x: -2, y: BOX_HALF.y + 0.002, z: 0 }));
      const hull = world.addBody({
        ...box1000({ x: 2, y: BOX_HALF.y + 0.002, z: 0 }),
        shapes: [{ kind: 'hull', points: corners(BOX_HALF), offset: ZERO }],
      });
      run(world, 1);
      const nativeY = world.getPose(native).position.y;
      expect(Math.abs(world.getPose(hull).position.y - nativeY)).toBeLessThan(0.001);
    });
  });

  it('places hull pieces of a compound where their points say (an L of two slabs)', async () => {
    await withWorld(pairs({ 'concrete|wood': [0.5, 0] }), (world) => {
      world.addBody(floor());
      const slab = (x0: number, x1: number, y0: number, y1: number) =>
        corners({ x: (x1 - x0) / 2, y: (y1 - y0) / 2, z: 0.2 }).map((p) => ({
          x: p.x + (x0 + x1) / 2,
          y: p.y + (y0 + y1) / 2,
          z: p.z,
        }));
      const body = world.addBody({
        ...box1000({ x: 0, y: 0.002, z: 0 }),
        shapes: [
          { kind: 'hull', points: slab(-0.5, 0.5, 0, 0.1), offset: ZERO },
          { kind: 'hull', points: slab(-0.5, -0.4, 0.1, 1), offset: ZERO },
        ],
        massProperties: {
          mass: 50,
          centerOfMass: { x: -0.2, y: 0.2, z: 0 },
          inertia: { x: 5, y: 5, z: 5 },
        },
      });
      // A beam cast down at x = 0.3 meets the bottom slab (top at 0.1 m), not the upright.
      const hit = world.castRay({ x: 0.3, y: 2, z: 0 }, { x: 0, y: -1, z: 0 }, 5);
      expect(hit?.body).toBe(body);
      expect(hit?.distance).toBeCloseTo(2 - 0.102, 2);
    });
  });

  it('lets a body that may sleep come to rest and keeps the state hash repeatable', async () => {
    const hashes: string[] = [];
    for (let i = 0; i < 2; i++) {
      await withWorld(pairs({ 'concrete|wood': [0.5, 0] }), (world) => {
        world.addBody(floor());
        const box = world.addBody({
          ...box1000({ x: 0, y: BOX_HALF.y + 0.3, z: 0 }),
          allowSleep: true,
        });
        run(world, 3);
        expect(world.getPose(box).position.y).toBeCloseTo(BOX_HALF.y, 2);
        expect(Math.hypot(...Object.values(world.getVelocity(box).linear))).toBeLessThan(1e-3);
        hashes.push(world.stateHash());
      });
    }
    expect(hashes[1]).toBe(hashes[0]);
  });
});

describe('Jolt backend: queries and determinism', () => {
  it('casts a ray and reports the first body and its distance', async () => {
    await withWorld(pairs({ 'concrete|wood': [0.5, 0] }), (world) => {
      world.addBody(floor());
      const box = world.addBody(box1000({ x: 2, y: BOX_HALF.y, z: 0 }));
      expect(world.castRay({ x: 0, y: 0.07, z: 0 }, { x: 1, y: 0, z: 0 }, 5)).toEqual({
        body: box,
        distance: expect.closeTo(1.4, 3),
      });
      expect(world.castRay({ x: 0, y: 0.07, z: 0 }, { x: -1, y: 0, z: 0 }, 5)).toBeUndefined();
    });
  });

  it('skips the bodies a filter rejects, so a beam sees the pallet behind a guide', async () => {
    await withWorld(
      pairs({ 'concrete|wood': [0.5, 0], 'steel-painted|wood': [0.3, 0.1] }),
      (world) => {
        world.addBody(floor());
        const wall = world.addBody(wallAt(0.5));
        const box = world.addBody(box1000({ x: 2, y: BOX_HALF.y, z: 0 }));
        const origin = { x: 0, y: 0.07, z: 0 };
        const direction = { x: 1, y: 0, z: 0 };
        expect(world.castRay(origin, direction, 5)?.body).toBe(wall);
        expect(world.castRay(origin, direction, 5, (body) => body === box)).toEqual({
          body: box,
          distance: expect.closeTo(1.4, 3),
        });
        expect(world.castRay(origin, direction, 1, (body) => body === box)).toBeUndefined();
      },
    );
  });

  it('gives the same state hash for the same inputs and a different one otherwise', async () => {
    const hashes: string[] = [];
    for (const speed of [0.25, 0.25, 0.26]) {
      await withWorld(
        pairs({ 'concrete|wood': [0.5, 0], 'steel-painted|wood': [0.3, 0.1] }),
        (world) => {
          world.addBody(floor());
          world.addBody(wallAt(0.5));
          const box = world.addBody(box1000({ x: -BOX_HALF.x, y: BOX_HALF.y, z: 0 }));
          world.setVelocity(box, { x: speed * 4, y: 0, z: 0 }, ZERO);
          run(world, 2);
          hashes.push(world.stateHash());
        },
      );
    }
    expect(hashes[0]).toBe(hashes[1]);
    expect(hashes[2]).not.toBe(hashes[0]);
  });
});

async function withWorld(
  lookup: ContactPairLookup,
  test: (world: PhysicsBackend) => void,
): Promise<void> {
  const world = await createJoltBackend({ contactPair: lookup });
  try {
    test(world);
  } finally {
    world.dispose();
  }
}

/** Contact table for tests, keyed "a|b" with the two material ids in alphabetical order. */
function pairs(values: Record<string, [number, number]>): ContactPairLookup {
  return (a, b) => {
    const value = values[[a, b].sort().join('|')];
    if (value === undefined) throw new Error(`no pair ${a}|${b}`);
    return { mu: value[0], restitution: value[1] };
  };
}

function run(world: PhysicsBackend, seconds: number): void {
  for (let i = 0; i < Math.round(seconds / DT); i++) world.step(DT);
}

function floor(): BodyDesc {
  return {
    motion: 'static',
    pose: pose({ x: 0, y: -0.05, z: 0 }),
    shapes: [box({ x: 10, y: 0.05, z: 10 })],
    material: 'concrete',
  };
}

function wallAt(x: number): BodyDesc {
  return {
    motion: 'static',
    pose: pose({ x: x + 0.05, y: 0.3, z: 0 }),
    shapes: [box({ x: 0.05, y: 0.3, z: 1 })],
    material: 'steel-painted',
  };
}

function box1000(position: Vec3): BodyDesc {
  const { x, y, z } = BOX_HALF;
  const inertia = (a: number, b: number) => (BOX_MASS * (4 * a * a + 4 * b * b)) / 12;
  return {
    motion: 'dynamic',
    pose: pose(position),
    shapes: [box(BOX_HALF)],
    material: 'wood',
    massProperties: {
      mass: BOX_MASS,
      centerOfMass: ZERO,
      inertia: { x: inertia(y, z), y: inertia(x, z), z: inertia(x, y) },
    },
    ccd: true,
  };
}

function rollerRow(
  world: PhysicsBackend,
  count: number,
  pitch: number,
  diameter: number,
): BodyHandle[] {
  return Array.from({ length: count }, (_, i) =>
    world.addBody({
      motion: 'kinematic',
      pose: pose({ x: i * pitch, y: 0, z: 0 }),
      shapes: [
        { kind: 'cylinder', radius: diameter / 2, halfLength: 0.5, axis: 'z', offset: ZERO },
      ],
      material: 'steel-zinc',
    }),
  );
}

function box(halfExtents: Vec3) {
  return { kind: 'box' as const, halfExtents, offset: ZERO };
}

function pose(position: Vec3) {
  return { position, rotation: { x: 0, y: 0, z: 0, w: 1 } };
}

function corners(half: Vec3): Vec3[] {
  const out: Vec3[] = [];
  for (const x of [-half.x, half.x])
    for (const y of [-half.y, half.y]) for (const z of [-half.z, half.z]) out.push({ x, y, z });
  return out;
}
