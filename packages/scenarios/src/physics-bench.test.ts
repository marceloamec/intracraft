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
