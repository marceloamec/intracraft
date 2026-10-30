import { type Instance, type Layout, secondsToSimTime } from '@intracraft/core-model';
import { createBuiltInRegistry, deriveConnections, instanceParams, num } from '@intracraft/library';
import { describe, expect, it } from 'vitest';
import { FlowSimulation } from './flow-simulation';
import { buildFlowNetwork } from './network';

const REGISTRY = createBuiltInRegistry();
const SEED = 7;
const REL_TOL = 0.01;

describe('2D flow: reference tests', () => {
  it('T1: a pallet crosses a 3 m roller conveyor from rest in 12.25 s (0.25 m/s, 0.5 m/s²)', () => {
    const sim = run(line(['flow.source', { maxPallets: 1 }], ['conveyor.roller'], ['flow.sink']), 60);
    const stats = sim.statistics()['roller1'];
    expect(stats?.['palletsProcessed']).toBe(1);
    expect(stats?.['meanTransitTime']).toBeCloseTo(12.25, 5);
  });

  it('T2: EUR long side with 150 mm gap reaches ≈ 667 pallets/h', () => {
    const sim = run(line(['flow.source', { interval: 1 }], ['conveyor.roller', { length: 12 }], ['flow.sink']), 7200);
    const perHour = sim.statistics()['sink1']?.['throughputPerHour'] ?? 0;
    expect(Math.abs(perHour - 3600 * (0.25 / 1.35)) / 666.7).toBeLessThan(REL_TOL);
  });
});

describe('2D flow: conveyors without accumulation', () => {
  it('hands a pallet over to the next conveyor without stopping', () => {
    const layout = line(['flow.source', { maxPallets: 1 }], ['conveyor.roller'], ['conveyor.roller'], ['flow.sink']);
    const stats = run(layout, 60).statistics();
    // 12.25 s on the first conveyor, then 3 m + 20 mm gap at 0.25 m/s.
    expect(stats['roller2']?.['meanTransitTime']).toBeCloseTo(12.08, 5);
    expect(stats['sink1']?.['meanCycleTime']).toBeCloseTo(24.33, 5);
  });

  it('stops the pallet with its front at the end when the output is not connected', () => {
    const sim = run(line(['flow.source', { maxPallets: 1 }], ['conveyor.roller']), 60);
    const [pallet] = sim.snapshot().pallets;
    expect(pallet?.front).toBeCloseTo(3, 6);
    expect(pallet?.speed).toBe(0);
    expect(sim.snapshot().status['roller1']).toBe('blocked');
    expect(sim.issues.map((i) => i.rule)).toContain('openEnd');
  });

  it('a blocked conveyor receives nothing: the one before it holds and the source waits', () => {
    const sim = run(line(['flow.source', { interval: 5 }], ['conveyor.roller'], ['conveyor.roller']), 600);
    const snapshot = sim.snapshot();
    // Each 3 m conveyor holds two EUR pallets; the front ones stand exactly at the end.
    const fronts = snapshot.pallets.map((p) => [p.carrier, round(p.front)]);
    expect(fronts).toHaveLength(4);
    expect(fronts).toContainEqual(['roller1', 3]);
    expect(fronts).toContainEqual(['roller2', 3]);
    expect(snapshot.pallets.every((p) => p.speed === 0 && p.front <= 3 + 1e-6)).toBe(true);
    expect(snapshot.status).toMatchObject({ roller1: 'blocked', roller2: 'blocked', source1: 'waiting' });
    expect(sim.statistics()['source1']?.['waitingTime']).toBeGreaterThan(500);
  });

  it('rejects pallets instead of waiting when the source must not block', () => {
    const layout = line(
      ['flow.source', { interval: 6, blockIfOccupied: false }],
      ['conveyor.roller'],
    );
    const stats = run(layout, 600).statistics()['source1'];
    expect(stats?.['palletsCreated']).toBe(2);
    expect(stats?.['palletsRejected']).toBeGreaterThan(90);
  });

  it('moves the pallets of coupled conveyors at the speed of the slower one', () => {
    const layout = line(['flow.source', { maxPallets: 1 }], ['conveyor.roller'], ['conveyor.roller', { nominalSpeed: 0.1 }], ['flow.sink']);
    const sim = new FlowSimulation(buildFlowNetwork(layout, REGISTRY), { seed: SEED });
    sim.runUntil(secondsToSimTime(9)); // the front has passed the decision point of roller1
    const [pallet] = sim.snapshot().pallets;
    expect(pallet?.carrier).toBe('roller1');
    expect(pallet?.speed).toBeCloseTo(0.1, 9);
  });

  it('counts the time a conveyor waits for a busy sink as blocked', () => {
    const layout = line(
      ['flow.source', { interval: 10 }],
      ['conveyor.roller'],
      ['flow.sink', { removalMode: 'afterTime', removalTime: 60 }],
    );
    const stats = run(layout, 3600).statistics();
    expect(stats['sink1']?.['throughputPerHour']).toBeCloseTo(60, -1);
    expect(stats['roller1']?.['blockedTime']).toBeGreaterThan(2000);
  });
});

describe('2D flow: invariants over a long irregular run', () => {
  it('never lets pallets overlap or run backwards', () => {
    const layout = line(
      ['flow.source', { generationMode: 'exponential', interval: 4 }],
      ['conveyor.roller', { length: 4 }],
      ['conveyor.roller', { length: 2.5, nominalSpeed: 0.2 }],
      ['conveyor.roller', { length: 6, accel: 0.3 }],
      ['flow.sink', { removalMode: 'afterTime', removalTime: 9 }],
    );
    const sim = new FlowSimulation(buildFlowNetwork(layout, REGISTRY), { seed: SEED });
    const lastFront = new Map<string, number>();
    for (let t = 0; t <= 4 * 3600; t += 0.7) {
      sim.runUntil(secondsToSimTime(t));
      const pallets = sim.snapshot().pallets;
      for (const [i, pallet] of pallets.entries()) {
        const ahead = pallets[i - 1];
        if (ahead?.carrier === pallet.carrier) expect(ahead.front - ahead.length).toBeGreaterThanOrEqual(pallet.front - 1e-6);
        const key = `${pallet.id}@${pallet.carrier}`;
        expect(pallet.front).toBeGreaterThanOrEqual((lastFront.get(key) ?? -Infinity) - 1e-6);
        lastFront.set(key, pallet.front);
      }
    }
    // The sink takes one pallet per 9 s plus the 1.22 m crossing from standstill (≈ 15 s per pallet).
    expect(sim.statistics()['sink1']?.['palletsReceived']).toBeGreaterThan(900);
  });
});

describe('2D flow: source, warnings and determinism', () => {
  it('creates no more than maxPallets', () => {
    const stats = run(line(['flow.source', { maxPallets: 3, interval: 1 }], ['conveyor.roller'], ['flow.sink']), 600).statistics();
    expect(stats['source1']?.['palletsCreated']).toBe(3);
    expect(stats['sink1']?.['palletsReceived']).toBe(3);
  });

  it('warns when the pallet orientation does not match the conveyor', () => {
    const sim = run(line(['flow.source', { maxPallets: 1 }], ['conveyor.chain'], ['flow.sink']), 60);
    expect(sim.issues.map((i) => i.rule)).toContain('palletOrientationMismatch');
  });

  it('holds pallets in front of objects without a 2D behaviour yet', () => {
    const registry = createBuiltInRegistry();
    registry.register({ ...REGISTRY.get('conveyor.roller'), id: 'conveyor.future', behaviors: { des: 'des.future', physics: 'phys.future' } });
    const layout = line(['flow.source', { maxPallets: 1 }], ['conveyor.roller'], ['conveyor.roller']);
    const future = { ...layout, instances: layout.instances.map((i) => (i.id === 'roller2' ? { ...i, definitionId: 'conveyor.future' } : i)) };
    const sim = new FlowSimulation(buildFlowNetwork(future, registry), { seed: SEED });
    sim.runUntil(secondsToSimTime(60));
    expect(sim.issues.map((i) => i.rule)).toContain('behaviorNotAvailable');
    expect(sim.snapshot().pallets[0]?.front).toBeCloseTo(3, 6);
  });

  it('gives the same results for the same seed and different ones for another seed', () => {
    const layout = line(['flow.source', { generationMode: 'exponential', interval: 8 }], ['conveyor.roller'], ['flow.sink']);
    const first = run(layout, 3600).statistics();
    expect(run(layout, 3600).statistics()).toEqual(first);
    const other = new FlowSimulation(buildFlowNetwork(layout, REGISTRY), { seed: SEED + 1 });
    other.runUntil(secondsToSimTime(3600));
    expect(other.statistics()['source1']?.['palletsCreated']).not.toBe(first['source1']?.['palletsCreated']);
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
    const values = instanceParams(definition, params);
    const instance: Instance = {
      id: `${prefix}${count}`,
      definitionId,
      definitionVersion: definition.version,
      name: '',
      position: { x, y: 0, z: 0 },
      yaw: 0,
      params: values,
      attributes: {},
    };
    x += num(values, 'length', definitionId === 'flow.source' ? 0.4 : 0) + 0.02;
    return instance;
  });
  const layout: Layout = { instances, connections: [] };
  return { ...layout, connections: deriveConnections(layout, REGISTRY) };
}

function run(layout: Layout, seconds: number): FlowSimulation {
  const sim = new FlowSimulation(buildFlowNetwork(layout, REGISTRY), { seed: SEED });
  sim.runUntil(secondsToSimTime(seconds));
  return sim;
}

function round(value: number): number {
  return Math.round(value * 1e6) / 1e6;
}
