import { type Instance, type Layout, type SimTime, secondsToSimTime } from '@intracraft/core-model';
import { createBuiltInRegistry, deriveConnections, instanceParams, num } from '@intracraft/library';
import { FlowSimulation, buildFlowNetwork } from '@intracraft/sim-des';
import { describe, expect, it } from 'vitest';
import { type PhysicsFlowSimulation, PhysicsFlowSimulation as Physics } from './physics-flow';

const REGISTRY = createBuiltInRegistry();
const SEED = 7;
const REL_TOL = 0.01;
/** 2D and 3D sensor edges must agree within ±50 ms (S06 P6, A11). */
const EDGE_TOL = 0.05;
/** Stop criterion of spike S1: the front within ±5 mm (ADR-0002). */
const STOP_TOL = 0.005;

describe('3D flow: reference tests', () => {
  it('T1: a pallet crosses a 3 m roller conveyor from rest in 12.25 s ±1 % (0.25 m/s, 0.5 m/s²)', async () => {
    await withRun(
      line(['flow.source', { maxPallets: 1 }], ['conveyor.roller'], ['flow.sink']),
      16,
      (sim) => {
        const stats = sim.statistics();
        expect(stats['roller1']?.['palletsProcessed']).toBe(1);
        expectWithin(stats['roller1']?.['meanTransitTime'], 12.25, REL_TOL);
        expect(stats['sink1']?.['palletsReceived']).toBe(1);
      },
    );
  });

  it('T1 on chains (plan B, EUR short side): the same 12.25 s ±1 %', async () => {
    const layout = line(
      ['flow.source', { maxPallets: 1, orientation: 'short' }],
      ['conveyor.chain'],
      ['flow.sink'],
    );
    await withRun(layout, 16, (sim) =>
      expectWithin(sim.statistics()['chain1']?.['meanTransitTime'], 12.25, REL_TOL),
    );
  });
});

describe('3D flow: conveyors without accumulation (ADR-0007 in 3D)', () => {
  it('hands a pallet over to the next conveyor without stopping (24.33 s to the sink, as in 2D)', async () => {
    const layout = line(
      ['flow.source', { maxPallets: 1 }],
      ['conveyor.roller'],
      ['conveyor.roller'],
      ['flow.sink'],
    );
    await withRun(layout, 30, (sim) =>
      expectWithin(sim.statistics()['sink1']?.['meanCycleTime'], 24.33, REL_TOL),
    );
  });

  it('stops the pallet with its front at the open end within ±5 mm and holds the conveyor', async () => {
    await withRun(line(['flow.source', { maxPallets: 1 }], ['conveyor.roller']), 20, (sim) => {
      const [pallet] = sim.snapshot().pallets;
      expect(Math.abs((pallet?.front ?? 0) - 3)).toBeLessThan(STOP_TOL);
      expect(Math.abs(pallet?.speed ?? 1)).toBeLessThan(0.002);
      expect(sim.snapshot().status['roller1']).toBe('blocked');
      expect(sim.issues.map((i) => i.rule)).toContain('openEnd');
    });
  });

  it('a blocked conveyor receives nothing: two pallets per 3 m conveyor, the front ones at the end', async () => {
    await withRun(
      line(['flow.source', { interval: 5 }], ['conveyor.roller'], ['conveyor.roller']),
      90,
      (sim) => {
        const snapshot = sim.snapshot();
        expect(snapshot.pallets).toHaveLength(4);
        for (const carrier of ['roller1', 'roller2']) {
          const fronts = snapshot.pallets.filter((p) => p.carrier === carrier).map((p) => p.front);
          expect(Math.abs(Math.max(...fronts) - 3)).toBeLessThan(STOP_TOL);
          expect(Math.abs(Math.min(...fronts) - 1.65)).toBeLessThan(STOP_TOL);
        }
        expect(snapshot.status).toMatchObject({
          roller1: 'blocked',
          roller2: 'blocked',
          source1: 'waiting',
        });
        expect(sim.physicsFigures().impacts).toBe(0);
      },
    );
  });

  it('a controller rule switches a conveyor off at a photocell: the pallet stops there', async () => {
    const layout = line(['flow.source', { maxPallets: 1 }], ['conveyor.roller'], ['flow.sink']);
    layout.instances.push(photoeye('photoeye1', 'roller1', 2));
    layout.rules = [
      {
        id: 'r1',
        when: [{ signal: 'photoeye1.detected', is: true }],
        then: [{ signal: 'roller1.run', value: false }],
      },
    ];
    await withRun(layout, 20, (sim) => {
      const [pallet] = sim.snapshot().pallets;
      // Detected when the front reaches 2 m; braking from 0.25 m/s at 0.5 m/s² adds 62.5 mm.
      expect(pallet?.front).toBeGreaterThan(2);
      expect(pallet?.front).toBeLessThan(2.08);
      expect(sim.snapshot().signals['roller1.run']).toBe(false);
      expect(sim.statistics()['sink1']?.['palletsReceived']).toBe(0);
    });
  });
});

describe('3D flow: photocells and determinism', () => {
  it('P6: every photocell edge of the 3D run lies within ±50 ms of the 2D edge', async () => {
    const layout = line(
      ['flow.source', { interval: 8, maxPallets: 3 }],
      ['conveyor.roller', { length: 4 }],
      ['flow.sink'],
    );
    // A beam at 400 mm crosses the load: one pulse per pallet.
    layout.instances.push(
      photoeye('photoeye1', 'roller1', 3, { sensorHeight: 0.4, reflectorHeight: 0.4 }),
    );
    const expected = edges2d(layout, 40, 'photoeye1');
    expect(expected).toHaveLength(6);
    await withRun(layout, 40, (sim) => {
      const edges = sim.takeSensorEdges().filter((e) => e.sensor === 'photoeye1');
      expect(edges.map((e) => e.detecting)).toEqual(expected.map((e) => e.detecting));
      for (const [i, edge] of edges.entries()) {
        expect(Math.abs(edge.time - (expected[i]?.time ?? 0)) / 1e6).toBeLessThan(EDGE_TOL);
      }
    });
  });

  it('the beam ignores conveyor parts and sees the pallet voids at the bottom-board height', async () => {
    const layout = line(['flow.source', { maxPallets: 1 }], ['conveyor.roller'], ['flow.sink']);
    // 60 mm: the beam crosses the blocks and the voids between them, as in 2D (T6).
    layout.instances.push(photoeye('photoeye1', 'roller1', 1.5));
    const expected = edges2d(layout, 16, 'photoeye1');
    expect(expected.length).toBeGreaterThan(2);
    await withRun(layout, 16, (sim) => {
      expect(sim.takeSensorEdges().filter((e) => e.detecting).length).toBe(
        expected.filter((e) => e.detecting).length,
      );
    });
  });

  it('gives the same state and statistics for the same layout and seed', async () => {
    const layout = line(
      ['flow.source', { interval: 7, generationMode: 'exponential' }],
      ['conveyor.roller'],
      ['conveyor.roller'],
      ['flow.sink'],
    );
    const results: string[] = [];
    for (let run = 0; run < 2; run++) {
      await withRun(layout, 40, (sim) =>
        results.push(sim.stateHash() + JSON.stringify(sim.statistics())),
      );
    }
    expect(results[0]).toBe(results[1]);
  });
});

describe('3D flow: lift stations (S12–S14)', () => {
  it('the C elevator lifts each pallet 4 m and returns: 30.0 s cycle ±1 %, as in 2D', async () => {
    await withRun(elevatorLayout(), 120, (sim) => {
      const stats = sim.statistics();
      expect(stats['sink1']?.['palletsReceived']).toBe(1);
      expectWithin(stats['elevator1']?.['meanCycleTime'], 30.03, REL_TOL);
      expect(sim.physicsFigures().fallenPallets).toBe(0);
    });
  });

  it('reports the deck height while lifting', async () => {
    await withRun(elevatorLayout(), 20, (sim) => {
      const height = sim.snapshot().heights['elevator1'] ?? 0;
      expect(height).toBeGreaterThan(0.6);
      expect(height).toBeLessThan(4.6);
      expect(sim.snapshot().status['elevator1']).toBe('lifting');
    });
  });

  it('a manual input table lifts the pallet the operator puts down (100 mm) to the line (600 mm)', async () => {
    const layout = line(
      ['flow.source', { maxPallets: 1, surfaceHeight: 0.1 }],
      ['conveyor.lift_table_manual', { flowDirection: 'in' }],
      ['conveyor.roller'],
      ['flow.sink'],
    );
    const expected = run2d(layout, 60)['sink1']?.['meanCycleTime'] ?? 0;
    await withRun(layout, 60, (sim) => {
      expect(sim.statistics()['sink1']?.['palletsReceived']).toBe(1);
      expectWithin(sim.statistics()['sink1']?.['meanCycleTime'], expected, REL_TOL);
    });
  });
});

describe('3D flow: gravity line (S15) and process station (S11)', () => {
  it('pallets roll to the end stop at the brake speed: impact ½·m·v² at 0.2 m/s, nothing falls', async () => {
    const layout = line(
      ['flow.source', { interval: 1, maxPallets: 3, surfaceHeight: 0.34 }],
      ['conveyor.roller', { surfaceHeight: 0.34 }],
      ['conveyor.gravity'],
      ['flow.sink', { removalMode: 'afterTime', removalTime: 60, surfaceHeight: 0.1 }],
    );
    await withRun(layout, 80, (sim) => {
      const impacts = sim.takeImpacts();
      const endStop = impacts.filter((i) => i.other === 'gravity1.endStop');
      expect(endStop.length).toBeGreaterThan(0);
      // EUR pallet with 600 kg load (≈ 631 kg) at 0.2 m/s: 12.6 J.
      expectWithin(endStop[0]?.energy, 12.6, 0.1);
      expect(Math.max(...impacts.map((i) => i.speed))).toBeLessThan(0.21);
      // The operator needs 60 s per pallet: one taken, the next waits at the end stop.
      expect(sim.statistics()['sink1']?.['palletsReceived']).toBe(1);
      expect(sim.snapshot().status['gravity1']).toBe('waiting');
      expect(sim.physicsFigures().fallenPallets).toBe(0);
    });
  });

  it('a process station adds its processing time to the cycle (30 s → 42.03 s, as in 2D)', async () => {
    const layout = line(
      ['flow.source', { maxPallets: 1 }],
      ['conveyor.roller'],
      ['flow.process_station', { processingTime: 30 }],
      ['conveyor.roller'],
      ['flow.sink'],
    );
    await withRun(layout, 80, (sim) => {
      const stats = sim.statistics()['process_station1'];
      expectWithin(stats?.['meanCycleTime'], 42.03, REL_TOL);
      expect(stats?.['palletsProcessed']).toBe(1);
      expect(sim.statistics()['sink1']?.['palletsReceived']).toBe(1);
    });
  });

  it('warns that turntables have no 3D behaviour yet', async () => {
    await withRun(
      line(['flow.source', { maxPallets: 1 }], ['conveyor.roller'], ['conveyor.roller_turntable']),
      1,
      (sim) => {
        expect(sim.issues.map((i) => i.rule)).toContain('behaviorNotAvailable3D');
      },
    );
  });
});

type Part = [definitionId: string, params?: Record<string, number | boolean | string>];

/** Objects placed one after another along +x, 20 mm apart, connected port to port. */
function line(...parts: Part[]): Layout {
  const counters = new Map<string, number>();
  let x = 0;
  const instances: Instance[] = parts.map(([definitionId, params = {}]) => {
    const instance = place(definitionId, params, x, counters);
    x += num(instance.params, 'length', definitionId === 'flow.source' ? 0.4 : 0) + 0.02;
    return instance;
  });
  return connected(instances);
}

/** Floor conveyor → C elevator (0.6 → 4.6 m) → mezzanine conveyor running back → sink. */
function elevatorLayout(): Layout {
  const counters = new Map<string, number>();
  const instances = [
    place('flow.source', { maxPallets: 1 }, -3.44, counters),
    place('conveyor.roller', {}, -3.02, counters),
    place('conveyor.elevator', {}, 0, counters),
    {
      ...place('conveyor.roller', {}, -0.02, counters),
      position: { x: -0.02, y: 4, z: 0 },
      yaw: Math.PI,
    },
    {
      ...place('flow.sink', {}, -3.04, counters),
      position: { x: -3.04, y: 4, z: 0 },
      yaw: Math.PI,
    },
  ];
  return connected(instances);
}

function place(
  definitionId: string,
  params: Record<string, number | boolean | string>,
  x: number,
  counters: Map<string, number>,
): Instance {
  const definition = REGISTRY.get(definitionId);
  const prefix = definitionId.split('.')[1] ?? definitionId;
  const count = (counters.get(prefix) ?? 0) + 1;
  counters.set(prefix, count);
  return {
    id: `${prefix}${count}`,
    definitionId,
    definitionVersion: definition.version,
    name: '',
    position: { x, y: 0, z: 0 },
    yaw: 0,
    params: instanceParams(definition, params),
    attributes: {},
  };
}

function connected(instances: Instance[]): Layout {
  const layout: Layout = { instances, connections: [] };
  return { ...layout, connections: deriveConnections(layout, REGISTRY) };
}

/** A retroreflective photocell across `host` at `s` (m from its entry). */
function photoeye(
  id: string,
  host: string,
  s: number,
  params: Record<string, number> = {},
): Instance {
  const definition = REGISTRY.get('sensor.photoeye');
  return {
    id,
    definitionId: definition.id,
    definitionVersion: definition.version,
    name: '',
    position: { x: 0, y: 0, z: 0 },
    yaw: 0,
    params: instanceParams(definition, { host, sensorS: s, reflectorS: s, ...params }),
    attributes: {},
  };
}

/** Output edges of a 2D sensor, event by event (exact times). */
function edges2d(
  layout: Layout,
  seconds: number,
  sensor: string,
): { time: SimTime; detecting: boolean }[] {
  const sim = new FlowSimulation(buildFlowNetwork(layout, REGISTRY), { seed: SEED });
  const end = secondsToSimTime(seconds);
  const edges: { time: SimTime; detecting: boolean }[] = [];
  let detecting = false;
  for (
    let next = sim.nextEventTime();
    next !== undefined && next <= end;
    next = sim.nextEventTime()
  ) {
    sim.runUntil(next);
    const now = sim.snapshot().status[sensor] === 'detecting';
    if (now !== detecting) edges.push({ time: next, detecting: now });
    detecting = now;
  }
  return edges;
}

function run2d(layout: Layout, seconds: number) {
  const sim = new FlowSimulation(buildFlowNetwork(layout, REGISTRY), { seed: SEED });
  sim.runUntil(secondsToSimTime(seconds));
  return sim.statistics();
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
  const sim = await Physics.create(layout, REGISTRY, { seed: SEED });
  try {
    sim.runUntil(secondsToSimTime(seconds));
    check(sim);
  } finally {
    sim.dispose();
  }
}
