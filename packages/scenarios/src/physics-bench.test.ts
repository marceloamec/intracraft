import { describe, expect, it } from 'vitest';
import {
  BENCH_PARAMETERS,
  BENCH_SURFACE_HEIGHT,
  MTBH_TYPES,
  PhysicsBench,
  benchDefaults,
} from './physics-bench';

describe('physics bench parameters', () => {
  it('only depends on parameters that exist', () => {
    const keys = new Set(BENCH_PARAMETERS.map((p) => p.key));
    for (const p of BENCH_PARAMETERS) {
      if (p.dependsOn !== undefined)
        expect(keys.has(p.dependsOn.split(/!?=/)[0] ?? ''), p.key).toBe(true);
    }
  });

  it('keeps every default inside its range', () => {
    for (const p of BENCH_PARAMETERS) {
      if (typeof p.default !== 'number') continue;
      if (p.min !== undefined) expect(p.default, p.key).toBeGreaterThanOrEqual(p.min);
      if (p.max !== undefined) expect(p.default, p.key).toBeLessThanOrEqual(p.max);
    }
  });

  it('defaults to the EUR pallet with cartons and 600 kg against a steel stop', () => {
    expect(benchDefaults()).toMatchObject({
      mtbhType: 'eur-wood-cartons',
      loadMass: 600,
      stopMaterial: 'steel-painted',
    });
  });
});

describe('physics bench layout', () => {
  it('lays 14 Ø89 rollers at 206.3 mm pitch on 3 m, a stop face at 2.8 m and the loaded EUR model', async () => {
    const bench = await PhysicsBench.create();
    try {
      const { rollers, stop, mtbh } = bench.layout;
      expect(rollers).toHaveLength(14);
      expect(rollers[0]?.y).toBeCloseTo(BENCH_SURFACE_HEIGHT - 0.0445, 9);
      expect(stop && stop.center.x - stop.halfExtents.x).toBeCloseTo(2.8, 9);
      expect(mtbh.visualAsset).toBe('asset.pallets.pallet_composition_a59');
      expect(mtbh.mass).toBeCloseTo(631.368, 3);
      expect(mtbh.length).toBeCloseTo(1.2, 3);
    } finally {
      bench.dispose();
    }
  });

  it.each(Object.keys(MTBH_TYPES))(
    'builds the %s pallet resting on the rollers at the start',
    async (mtbhType) => {
      const bench = await PhysicsBench.create({ mtbhType, accel: 0.1 });
      try {
        const frame = bench.run(0.2);
        expect(frame.pallet.pose.position.y).toBeCloseTo(BENCH_SURFACE_HEIGHT, 2);
        expect(frame.tilt).toBeLessThan(0.01);
        expect(frame.fallen).toBe(false);
      } finally {
        bench.dispose();
      }
    },
  );

  it('runs the pallet off the conveyor end when there is no stop', async () => {
    const bench = await PhysicsBench.create({
      stopEnabled: false,
      conveyorLength: 1.5,
      speed: 0.5,
      accel: 2,
    });
    try {
      expect(bench.layout.stop).toBeUndefined();
      expect(bench.run(5).fallen).toBe(true);
    } finally {
      bench.dispose();
    }
  });
});

describe('physics bench variants', () => {
  it('lays three chain strands at a 1050 mm gauge and turns the EUR pallet across them', async () => {
    const bench = await PhysicsBench.create({ deck: 'chains', palletSideAlongTravel: 'short' });
    try {
      const { rollers, strands, mtbh } = bench.layout;
      expect(rollers).toHaveLength(0);
      expect(strands.map((s) => s.z)).toEqual([-0.525, 0, 0.525]);
      expect(mtbh.start.yaw).toBeCloseTo(Math.PI / 2, 12);
      expect(bench.frame().front).toBeCloseTo(0.05 + 0.8, 2);
      expect(bench.run(0.2).pallet.pose.position.y).toBeCloseTo(BENCH_SURFACE_HEIGHT, 2);
    } finally {
      bench.dispose();
    }
  });

  it('carries the strands on small rollers for chain plan A', async () => {
    const bench = await PhysicsBench.create({
      deck: 'chainsPlanA',
      palletSideAlongTravel: 'short',
    });
    try {
      const { rollers, strands } = bench.layout;
      expect(strands).toHaveLength(3);
      expect(rollers.length).toBeGreaterThan(3 * 70);
      expect(new Set(rollers.map((r) => r.z))).toEqual(new Set([-0.525, 0, 0.525]));
    } finally {
      bench.dispose();
    }
  });

  it('adds two guide rails and a funnel whose mouth captures clearance + length · tan(angle)', async () => {
    const bench = await PhysicsBench.create({
      guidesEnabled: true,
      guideClearance: 0.01,
      funnelAngle: (15 * Math.PI) / 180,
      funnelLength: 0.5,
    });
    try {
      const { guides, mtbh } = bench.layout;
      expect(guides).toHaveLength(4);
      const rails = guides.filter((g) => g.yaw === 0);
      // Inner faces at half the pallet width (the model is 798 mm wide) + clearance.
      const inner = mtbh.width / 2 + 0.01;
      expect(rails.map((g) => Math.abs(g.center.z) - g.halfExtents.z)).toEqual([
        expect.closeTo(inner, 9),
        expect.closeTo(inner, 9),
      ]);
      expect(funnelMouth(bench.layout.guides)).toBeCloseTo(inner + 0.5 * Math.tan(Math.PI / 12), 9);
    } finally {
      bench.dispose();
    }
  });

  it('starts the pallet off-centre and turned when asked', async () => {
    const yaw = (5 * Math.PI) / 180;
    const bench = await PhysicsBench.create({ startOffsetZ: 0.05, startYaw: yaw });
    try {
      const frame = bench.frame();
      expect(frame.pallet.pose.position.z).toBeCloseTo(0.05, 6);
      expect(frame.yaw).toBeCloseTo(yaw, 6);
    } finally {
      bench.dispose();
    }
  });

  it('turns the platform 90° with the T3 profile and holds the pallet on its locked rollers', async () => {
    const bench = await PhysicsBench.create({ turntable: true, conveyorLength: 1.4 });
    try {
      expect(bench.layout.platform?.center.x).toBeCloseTo(0.7, 9);
      const start = bench.frame();
      expect(start.pallet.pose.position.x).toBeCloseTo(0.7, 6);
      const frame = bench.run(0.5 + 6.5 + 0.2);
      expect(frame.platformAngle).toBeCloseTo(Math.PI / 2, 2);
      expect(frame.rollerAngle).toBe(0);
    } finally {
      bench.dispose();
    }
  });

  it('builds 20 lanes of 14 rollers with a pallet on the first 10', async () => {
    const bench = await PhysicsBench.create({ lanes: 20, loadedLanes: 10, stopEnabled: false });
    try {
      expect(bench.layout.laneOffsets).toHaveLength(20);
      expect(bench.bodyCount).toEqual({ rollers: 280, pallets: 10 });
    } finally {
      bench.dispose();
    }
  });
});

/** Distance from the lane axis to the funnel's upstream end (inner face). */
function funnelMouth(
  guides: { center: { z: number }; halfExtents: { x: number; z: number }; yaw: number }[],
) {
  const plate = guides.find((g) => g.yaw !== 0 && g.center.z > 0);
  if (plate === undefined) throw new Error('No funnel plate');
  // Inner face end: centre − normal · t/2 + direction · (−length/2), direction (cos, −sin), normal (sin, cos).
  const c = Math.cos(plate.yaw);
  const s = Math.sin(plate.yaw);
  return plate.center.z - c * plate.halfExtents.z + s * plate.halfExtents.x;
}
