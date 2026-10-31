import { type Instance, type Layout, type Vec3, secondsToSimTime } from '@intracraft/core-model';
import { createBuiltInRegistry, deriveConnections, instanceParams } from '@intracraft/library';
import { describe, expect, it } from 'vitest';
import { FlowSimulation } from './flow-simulation';
import { buildFlowNetwork } from './network';

const REGISTRY = createBuiltInRegistry();
const SEED = 5;
const OPERATOR = { removalMode: 'afterTime', removalTime: 60 };

describe('2D gravity roller line (S15, ADR-0014)', () => {
  it('T10: a pallet entering at 0.2 m/s reaches the end stop 24 s after it is fully on the line', () => {
    const stats = run(gravityLine({ maxPallets: 1 }), 120).statistics()['gravity1'] ?? {};
    expect(stats['palletsDelivered']).toBe(1);
    expect(stats['meanTransitTime']).toBeCloseTo((6 - 1.2) / 0.2, 6);
  });

  it('T11: an operator taking 60 s per pallet sets 60 pallets/h; the line fills to 4 and blocks the feeder', () => {
    const sim = run(gravityLine({ interval: 1 }, OPERATOR), 4 * 3600);
    const stats = sim.statistics();
    expect(Math.abs((stats['sink1']?.['throughputPerHour'] ?? 0) - 60) / 60).toBeLessThan(0.01);
    expect(stats['gravity1']?.['maxQueue']).toBe(4);
    expect(stats['gravity1']?.['fullShare']).toBeGreaterThan(0.9);
    expect(sim.snapshot().status['roller1']).toBe('blocked');
    expect(sim.snapshot().status['gravity1']).toBe('blocked');
  });

  it('holds the second pallet at the separator (4.7 m) while the first waits in the pick-up position', () => {
    const sim = run(gravityLine({ interval: 1 }, OPERATOR), 600);
    const fronts = sim
      .snapshot()
      .pallets.filter((p) => p.carrier === 'gravity1')
      .map((p) => p.front)
      .sort((a, b) => b - a);
    expect(fronts).toEqual([
      expect.closeTo(6, 6),
      expect.closeTo(4.7, 6),
      expect.closeTo(3.5, 6),
      expect.closeTo(2.3, 6),
    ]);
  });

  it('startDelay = 0: the waiting queue starts as one block when the separator opens', () => {
    const gaps = gapsAfterRelease(0, 0.5);
    expect(gaps.length).toBeGreaterThanOrEqual(2);
    for (const gap of gaps) expect(gap).toBeCloseTo(0, 6);
  });

  it('startDelay = 2 s: a gap of brake speed × delay = 0.4 m opens between rolling pallets', () => {
    const [first] = gapsAfterRelease(2, 3.5);
    expect(first).toBeCloseTo(0.2 * 2, 6);
  });

  it('starts a pallet from rest with a = g·(slope − Crr) = 0.196 m/s² up to the brake speed', () => {
    // The released pallet covers v²/2a = 0.102 m while accelerating, then rolls at 0.2 m/s.
    const sim = afterRelease(1, 0.5);
    const released = sim.snapshot().pallets.find((p) => p.carrier === 'gravity1' && p.front > 4.7 && p.front < 6 - 1e-6);
    expect(released?.speed).toBeCloseTo(9.81 * 0.02 * 0.5, 6);
    expect(released?.accel).toBeCloseTo(9.81 * 0.02, 9);
  });

  it('warns when OUT is not connected or does not lead to a sink', () => {
    expect(run(gravityLine({ maxPallets: 1 }, {}, 'none'), 10).issues.map((i) => i.rule)).toContain('noExit');
    expect(run(gravityLine({ maxPallets: 1 }, {}, 'conveyor'), 10).issues.map((i) => i.rule)).toContain('exitNotSink');
  });

  it('gives the same statistics with the same seed', () => {
    const layout = gravityLine({ interval: 5, generationMode: 'exponential' }, { ...OPERATOR, removalTime: 30 });
    expect(run(layout, 3600).statistics()).toEqual(run(layout, 3600).statistics());
  });
});

type Params = Record<string, number | boolean | string>;

/** Source → roller (0.34 m) → gravity line (6 m, 4 %) → operator (sink at 0.1 m), along +x. */
function gravityLine(source: Params, operator: Params = {}, exit: 'sink' | 'none' | 'conveyor' = 'sink', line: Params = {}): Layout {
  const instances = [
    place('flow.source', { maxPallets: 0, ...source, surfaceHeight: 0.34 }, { x: 0, y: 0, z: 0 }),
    place('conveyor.roller', { surfaceHeight: 0.34 }, { x: 0.42, y: 0, z: 0 }),
    place('conveyor.gravity', line, { x: 3.44, y: 0, z: 0 }),
  ];
  if (exit === 'sink') instances.push(place('flow.sink', { ...operator, surfaceHeight: 0.1 }, { x: 9.46, y: 0, z: 0 }));
  if (exit === 'conveyor') instances.push(place('conveyor.roller', { surfaceHeight: 0.2 }, { x: 9.46, y: 0, z: 0 }));
  const counters = new Map<string, number>();
  const numbered = instances.map((instance) => {
    const count = (counters.get(instance.id) ?? 0) + 1;
    counters.set(instance.id, count);
    return { ...instance, id: `${instance.id}${count}` };
  });
  const layout: Layout = { instances: numbered, connections: [] };
  return { ...layout, connections: deriveConnections(layout, REGISTRY) };
}

function place(definitionId: string, params: Params, position: Vec3): Instance {
  const definition = REGISTRY.get(definitionId);
  return {
    id: definitionId.split('.')[1] ?? definitionId,
    definitionId,
    definitionVersion: definition.version,
    name: '',
    position,
    yaw: 0,
    params: instanceParams(definition, params),
    attributes: {},
  };
}

function run(layout: Layout, seconds: number): FlowSimulation {
  const sim = new FlowSimulation(buildFlowNetwork(layout, REGISTRY), { seed: SEED });
  sim.runUntil(secondsToSimTime(seconds));
  return sim;
}

/**
 * Fills the line, waits for the operator to take the pallet at the end stop, then runs until
 * `seconds` after the separator opened (separatorTime = 3 s).
 */
function afterRelease(startDelay: number, seconds: number): FlowSimulation {
  const sim = new FlowSimulation(buildFlowNetwork(gravityLine({ interval: 1 }, OPERATOR, 'sink', { startDelay }), REGISTRY), {
    seed: SEED,
  });
  // The line is full well before the second hand-over (60 s after the first).
  sim.runUntil(secondsToSimTime(100));
  const received = () => sim.statistics()['sink1']?.['palletsReceived'] ?? 0;
  const before = received();
  for (;;) {
    const next = sim.nextEventTime();
    if (next === undefined) throw new Error('No hand-over');
    sim.runUntil(next);
    if (received() > before) break;
  }
  sim.runUntil(sim.now + secondsToSimTime(3 + seconds));
  return sim;
}

/** Gaps between consecutive rolling pallets (m), front first, `seconds` after the separator opened. */
function gapsAfterRelease(startDelay: number, seconds: number): number[] {
  const sim = afterRelease(startDelay, seconds);
  const rolling = sim
    .snapshot()
    .pallets.filter((p) => p.carrier === 'gravity1' && p.speed > 0)
    .sort((a, b) => b.front - a.front);
  return rolling.slice(1).map((p, i) => (rolling[i]?.front ?? 0) - (rolling[i]?.length ?? 0) - p.front);
}

