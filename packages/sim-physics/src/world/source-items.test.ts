import { builtInCatalog } from '@intracraft/asset-catalog';
import { type Instance, type Layout, secondsToSimTime } from '@intracraft/core-model';
import { createBuiltInRegistry, deriveConnections, instanceParams, num } from '@intracraft/library';
import { FlowSimulation, buildFlowNetwork } from '@intracraft/sim-des';
import { describe, expect, it } from 'vitest';
import { sourceMtbh } from './pallet-body';
import { PhysicsFlowSimulation } from './physics-flow';

const REGISTRY = createBuiltInRegistry();
const SEED = 7;
const REL_TOL = 0.01;
const CRATE = 'asset.containers.crate_blue_a01';
const PLASTIC_PALLET = 'asset.pallets.pallet_rough_black_a02';
const BOX = 'asset.boxes.box_cardboard_a01';

describe('3D source items (S09 §1.1)', () => {
  it('a crate alone crosses 3 m of rollers in 12.25 s ±1 % and reaches the sink', async () => {
    const source = {
      maxPallets: 1,
      palletModel: 'none',
      loadKind: 'asset',
      loadAsset: CRATE,
      loadMass: 20,
    };
    await withRun(line(['flow.source', source], ['conveyor.roller'], ['flow.sink']), 1, (sim) => {
      const [crate] = sim.snapshot().pallets;
      expect(crate?.length).toBeCloseTo(0.605, 3);
      expect(Object.values(sim.snapshot().looks)[0]).toEqual({
        load: { asset: CRATE, placement: expect.anything() },
      });
      sim.runUntil(secondsToSimTime(16));
      const stats = sim.statistics();
      expectWithin(stats['roller1']?.['meanTransitTime'], 12.25, REL_TOL);
      expect(stats['sink1']?.['palletsReceived']).toBe(1);
    });
  });

  it('a plastic pallet with a catalogue box reaches the sink with the 2D time ±1 %, nothing falls', async () => {
    const source = {
      maxPallets: 1,
      palletModel: PLASTIC_PALLET,
      loadKind: 'asset',
      loadAsset: BOX,
      loadMass: 300,
    };
    const layout = line(
      ['flow.source', source],
      ['conveyor.roller'],
      ['conveyor.roller'],
      ['flow.sink'],
    );
    const sim2d = new FlowSimulation(buildFlowNetwork(layout, REGISTRY), { seed: SEED });
    sim2d.runUntil(secondsToSimTime(40));
    const expected = sim2d.statistics()['sink1']?.['meanCycleTime'] ?? 0;
    await withRun(layout, 40, (sim) => {
      expect(sim.statistics()['sink1']?.['palletsReceived']).toBe(1);
      expectWithin(sim.statistics()['sink1']?.['meanCycleTime'], expected, REL_TOL);
      expect(sim.physicsFigures().fallenPallets).toBe(0);
    });
  });

  it('tells the viewer which models to draw: the type model for auto, both models for a loaded pallet', async () => {
    const layout = line(
      ['flow.source', { maxPallets: 1 }],
      ['conveyor.roller'],
      [
        'flow.source',
        { maxPallets: 1, palletModel: PLASTIC_PALLET, loadKind: 'asset', loadAsset: BOX },
      ],
      ['conveyor.roller'],
    );
    await withRun(layout, 1, (sim) => {
      const looks = Object.values(sim.snapshot().looks);
      expect(looks).toContainEqual(
        expect.objectContaining({
          pallet: { asset: 'asset.pallets.pallet_block_a07', placement: expect.anything() },
        }),
      );
      const loaded = looks.find((l) => l.pallet?.asset === PLASTIC_PALLET);
      expect(loaded?.load?.asset).toBe(BOX);
      // The box stands on the pallet: its underside at the pallet top.
      const palletTop = builtInCatalog.get(PLASTIC_PALLET).dimensions.y;
      const boxBottom =
        (loaded?.load?.placement.translation.y ?? 0) + builtInCatalog.get(BOX).bounds.min.y;
      expect(boxBottom).toBeCloseTo(palletTop, 3);
    });
  });

  it('weighs pallet model + load mass and keeps the load above the pallet', () => {
    const source = {
      palletModel: PLASTIC_PALLET,
      loadKind: 'asset',
      loadAsset: BOX,
      loadMass: 300,
    };
    const node = buildFlowNetwork(line(['flow.source', source], ['conveyor.roller']), REGISTRY)
      .nodes[0];
    if (node?.kind !== 'source') throw new Error('no source');
    const body = sourceMtbh(node);
    const palletMass = builtInCatalog.get(PLASTIC_PALLET).physics?.mass ?? 0;
    expect(body.massProperties.mass).toBeCloseTo(palletMass + 300, 6);
    expect(body.massProperties.centerOfMass.y).toBeGreaterThan(
      builtInCatalog.get(PLASTIC_PALLET).dimensions.y,
    );
    expect(body.material).toBe('plastic');
  });
});

type Part = [definitionId: string, params?: Record<string, number | boolean | string>];

/** Objects placed one after another along +x, 20 mm apart, connected port to port. */
function line(...parts: Part[]): Layout {
  const counters = new Map<string, number>();
  let x = 0;
  const instances: Instance[] = parts.map(([definitionId, params = {}]) => {
    const definition = REGISTRY.get(definitionId);
    const prefix = definitionId.split('.')[1] ?? definitionId;
    const count = (counters.get(prefix) ?? 0) + 1;
    counters.set(prefix, count);
    const instance: Instance = {
      id: `${prefix}${count}`,
      definitionId,
      definitionVersion: definition.version,
      name: '',
      position: { x, y: 0, z: 0 },
      yaw: 0,
      params: instanceParams(definition, params),
      attributes: {},
    };
    x += num(instance.params, 'length', definitionId === 'flow.source' ? 0.4 : 0) + 0.02;
    return instance;
  });
  const layout: Layout = { instances, connections: [] };
  return { ...layout, connections: deriveConnections(layout, REGISTRY) };
}

function expectWithin(value: number | undefined, expected: number, relative: number): void {
  expect(value).toBeDefined();
  expect(Math.abs((value ?? 0) - expected) / expected).toBeLessThan(relative);
}

async function withRun(
  layout: Layout,
  seconds: number,
  check: (sim: PhysicsFlowSimulation) => void,
): Promise<void> {
  const sim = await PhysicsFlowSimulation.create(layout, REGISTRY, { seed: SEED });
  try {
    sim.runUntil(secondsToSimTime(seconds));
    check(sim);
  } finally {
    sim.dispose();
  }
}
