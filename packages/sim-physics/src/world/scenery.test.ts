import { builtInCatalog } from '@intracraft/asset-catalog';
import { assetColliders } from '@intracraft/asset-catalog/colliders';
import { GRAVITY, type Instance, type Layout, secondsToSimTime } from '@intracraft/core-model';
import { createBuiltInRegistry, instanceParams } from '@intracraft/library';
import { buildFlowNetwork } from '@intracraft/sim-des';
import { describe, expect, it } from 'vitest';
import { PhysicsFlowSimulation } from './physics-flow';

const REGISTRY = createBuiltInRegistry();
const SEED = 7;
const DT = 1 / 120;
const BOX = 'asset.boxes.box_cardboard_a01';
const RACK = 'asset.racks.rack_large_empty_a01';
/** Rest criterion of S17: the base on its support within ±5 mm. */
const REST_TOL = 0.005;

describe('3D scenery objects (S17)', () => {
  it('drop: a box released 1 m above the floor touches it after √(2h/g) = 0.452 s (± one step)', async () => {
    await withSim(
      [scenery('asset1', BOX, 'drop', { x: 0, y: 0, z: 0 }, { dropHeight: 1 })],
      (sim) => {
        let touch: number | undefined;
        for (let i = 0; i < 120 && touch === undefined; i++) {
          sim.step();
          if (baseOf(sim, 'asset1', BOX) < 0.003) touch = sim.now / 1e6;
        }
        expect(Math.abs((touch ?? 0) - Math.sqrt(2 / GRAVITY))).toBeLessThanOrEqual(DT + 1e-9);
        sim.runUntil(secondsToSimTime(3));
        expect(Math.abs(baseOf(sim, 'asset1', BOX))).toBeLessThan(REST_TOL);
      },
    );
  });

  it('rest: a box left on the floor moves less than 1 mm in 2 s', async () => {
    await withSim([scenery('asset1', BOX, 'rest', { x: 1, y: 0, z: 2 })], (sim) => {
      sim.step();
      const start = poseOf(sim, 'asset1').position;
      sim.runUntil(secondsToSimTime(2));
      const end = poseOf(sim, 'asset1').position;
      expect(Math.hypot(end.x - start.x, end.y - start.y, end.z - start.z)).toBeLessThan(0.001);
      expect(Math.abs(baseOf(sim, 'asset1', BOX))).toBeLessThan(REST_TOL);
    });
  });

  it('fixed: a rack never moves, and a box dropped above it stays on the shelf it falls on', async () => {
    const shelf = assetColliders(RACK)[0] ?? [];
    const cx = (Math.min(...shelf.map((p) => p.x)) + Math.max(...shelf.map((p) => p.x))) / 2;
    const cz = (Math.min(...shelf.map((p) => p.z)) + Math.max(...shelf.map((p) => p.z))) / 2;
    const top = builtInCatalog.get(RACK).bounds.max.y;
    const expected = highestTopUnder(RACK, cx, cz);
    const objects = [
      scenery('rack', RACK, 'fixed', { x: 0, y: 0, z: 0 }),
      scenery('box', BOX, 'drop', { x: cx, y: top, z: cz }, { dropHeight: 0.3 }),
    ];
    await withSim(objects, (sim) => {
      sim.runUntil(secondsToSimTime(4));
      expect(sim.snapshot().objects['rack']).toBeUndefined();
      expect(Math.abs(baseOf(sim, 'box', BOX) - expected)).toBeLessThan(REST_TOL);
    });
  });

  it('gives the same object poses for the same layout and seed', async () => {
    const objects = [
      scenery('a', BOX, 'drop', { x: 0, y: 0, z: 0 }, { dropHeight: 0.5 }),
      scenery(
        'b',
        'asset.drums.drum_steel_a01',
        'drop',
        { x: 0.1, y: 1.2, z: 0.05 },
        { dropHeight: 0.2 },
      ),
    ];
    const poses: unknown[] = [];
    for (let i = 0; i < 2; i++) {
      await withSim(objects, (sim) => {
        sim.runUntil(secondsToSimTime(3));
        poses.push(sim.snapshot().objects);
      });
    }
    expect(poses[1]).toEqual(poses[0]);
  });

  it('is ignored by the 2D model without a warning', () => {
    const network = buildFlowNetwork(
      layoutOf([scenery('asset1', BOX, 'rest', { x: 0, y: 0, z: 0 })]),
      REGISTRY,
    );
    expect(network.nodes.map((n) => n.id)).not.toContain('asset1');
    expect(network.issues).toEqual([]);
  });

  it('live spawn: an object dropped into a running world falls to the floor and is reported', async () => {
    await withSim([], (sim) => {
      sim.runUntil(secondsToSimTime(1));
      const id = sim.spawnObject({
        asset: BOX,
        placement: 'drop',
        dropHeight: 1,
        position: { x: 2, y: 0, z: 0 },
        yaw: 0,
      });
      sim.runUntil(secondsToSimTime(3));
      expect(Math.abs(baseOf(sim, id, BOX))).toBeLessThan(REST_TOL);
    });
  });

  it('rejects a live object of an unknown model', async () => {
    await withSim([], (sim) => {
      expect(() =>
        sim.spawnObject({
          asset: 'asset.nope.x',
          placement: 'rest',
          dropHeight: 1,
          position: { x: 0, y: 0, z: 0 },
          yaw: 0,
        }),
      ).toThrow(/unknown asset/i);
    });
  });
});

function scenery(
  id: string,
  asset: string,
  placement: 'drop' | 'rest' | 'fixed',
  position: { x: number; y: number; z: number },
  extra: Record<string, number | string> = {},
): Instance {
  const definition = REGISTRY.get('scenery.asset');
  return {
    id,
    definitionId: definition.id,
    definitionVersion: definition.version,
    name: id,
    position,
    yaw: 0,
    params: instanceParams(definition, { asset, placement, ...extra }),
    attributes: {},
  };
}

function layoutOf(instances: Instance[]): Layout {
  return { instances, connections: [] };
}

async function withSim(
  instances: Instance[],
  check: (sim: PhysicsFlowSimulation) => void,
): Promise<void> {
  const sim = await PhysicsFlowSimulation.create(layoutOf(instances), REGISTRY, { seed: SEED });
  try {
    check(sim);
  } finally {
    sim.dispose();
  }
}

function poseOf(sim: PhysicsFlowSimulation, id: string) {
  const view = sim.snapshot().objects[id];
  if (view === undefined) throw new Error(`no pose for ${id}`);
  return view.pose;
}

/** Height of the object's underside (upright objects: origin height + bounds.min.y). */
function baseOf(sim: PhysicsFlowSimulation, id: string, asset: string): number {
  return poseOf(sim, id).position.y + builtInCatalog.get(asset).bounds.min.y;
}

/** Top of the highest collision piece of `asset` whose footprint contains (x, z). */
function highestTopUnder(asset: string, x: number, z: number): number {
  let best = 0;
  for (const piece of assetColliders(asset)) {
    const xs = piece.map((p) => p.x);
    const zs = piece.map((p) => p.z);
    if (x < Math.min(...xs) || x > Math.max(...xs) || z < Math.min(...zs) || z > Math.max(...zs))
      continue;
    best = Math.max(best, ...piece.map((p) => p.y));
  }
  return best;
}
