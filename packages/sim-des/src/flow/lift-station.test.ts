import { type Instance, type Layout, US_PER_S, type Vec3, secondsToSimTime } from '@intracraft/core-model';
import { createBuiltInRegistry, deriveConnections, instanceParams } from '@intracraft/library';
import { describe, expect, it } from 'vitest';
import { FlowSimulation } from './flow-simulation';
import { buildFlowNetwork } from './network';

const REGISTRY = createBuiltInRegistry();
const SEED = 11;
/** Up 0.6 m at 0.05 m/s and 0.2 m/s²; down at 0.07 m/s (S13 §1). */
const T8_UP = 0.6 / 0.05 + 0.05 / 0.2;
const T8_DOWN = 0.6 / 0.07 + 0.07 / 0.2;
/** 4 m at 0.5 m/s and 0.5 m/s² (S14 §1). */
const T9_MOVE = 4 / 0.5 + 0.5 / 0.5;
/** Manual table: 0.5 m up at 0.05 m/s, down at 0.07 m/s, 0.2 m/s² (S12 §1). */
const MANUAL_UP = 0.5 / 0.05 + 0.05 / 0.2;
const MANUAL_DOWN = 0.5 / 0.07 + 0.07 / 0.2;

describe('2D lift stations (S12–S14, ADR-0013)', () => {
  describe('vertical moves', () => {
    it('T8: the scissor table lifts 0.6 → 1.2 m in 12.25 s and goes back down empty in 8.92 s', () => {
      const steps = timeline(scissorLine({ maxPallets: 1 }), 'scissor_lift1', 120);
      expect(durations(steps, 'lifting')).toEqual([expect.closeTo(T8_UP, 6)]);
      expect(durations(steps, 'lowering')).toEqual([expect.closeTo(T8_DOWN, 6)]);
      expect(T8_UP).toBeCloseTo(12.25, 9);
    });

    it('lowers with the pallet and lifts back empty when the exit is below the entry', () => {
      const steps = timeline(
        scissorLine({ maxPallets: 1 }, { entryHeight: 1.2, exitHeight: 0.6 }),
        'scissor_lift1',
        120,
      );
      expect(steps.map(([, status]) => status).filter((s) => s === 'lifting' || s === 'lowering')).toEqual([
        'lowering',
        'lifting',
      ]);
      expect(durations(steps, 'lowering')).toEqual([expect.closeTo(T8_DOWN, 6)]);
      expect(durations(steps, 'lifting')).toEqual([expect.closeTo(T8_UP, 6)]);
    });

    it('T9: the elevator moves 0.6 → 4.6 m in 9 s and back down empty in 9 s', () => {
      const steps = timeline(elevatorLine('far', { maxPallets: 1 }), 'elevator1', 120);
      expect(durations(steps, 'lifting')).toEqual([expect.closeTo(T9_MOVE, 6)]);
      expect(durations(steps, 'lowering')).toEqual([expect.closeTo(T9_MOVE, 6)]);
      expect(T9_MOVE).toBe(9);
    });
  });

  describe('manual lift table', () => {
    it('input: lifts the operator pallet in 10.25 s, lowers empty in 7.49 s and feeds the line', () => {
      const layout = manualInput({ maxPallets: 1 });
      const steps = timeline(layout, 'lift_table_manual1', 120);
      expect(durations(steps, 'lifting')).toEqual([expect.closeTo(MANUAL_UP, 6)]);
      expect(durations(steps, 'lowering')).toEqual([expect.closeTo(MANUAL_DOWN, 6)]);
      expect(run(layout, 120).statistics()['sink1']?.['palletsReceived']).toBe(1);
    });

    it('output: an operator taking 60 s per pallet sets the rate (at most 60 pallets/h) and blocks the table', () => {
      const sim = run(manualOutput({ interval: 10 }, { removalMode: 'afterTime', removalTime: 60 }), 4 * 3600);
      const perHour = sim.statistics()['sink1']?.['throughputPerHour'] ?? 0;
      // The next pallet waits at the deck end and needs 1.2 m from rest (0.25 m/s, 0.5 m/s²) to clear it.
      const handOver = 1.2 / 0.25 + 0.25 / (2 * 0.5);
      expect(perHour).toBeLessThanOrEqual(60);
      expect(Math.abs(perHour - 3600 / (60 + handOver)) / (3600 / (60 + handOver))).toBeLessThan(0.01);
      expect(sim.statistics()['lift_table_manual1']?.['blockedShare']).toBeGreaterThan(0.5);
    });

    it('output: the pallet runs onto the table backwards and leaves through OP at the low height', () => {
      const sim = new FlowSimulation(buildFlowNetwork(manualOutput({ maxPallets: 1 }), REGISTRY), { seed: SEED });
      let lowest = Infinity;
      for (let t = 0; t < 80; t += 0.25) {
        sim.runUntil(secondsToSimTime(t));
        const snapshot = sim.snapshot();
        lowest = Math.min(lowest, snapshot.heights['lift_table_manual1'] ?? Infinity);
        for (const pallet of snapshot.pallets.filter((p) => p.carrier === 'lift_table_manual1')) {
          expect(pallet.heading).toBeCloseTo(Math.PI, 9);
        }
      }
      expect(lowest).toBeCloseTo(0.1, 9);
      expect(sim.statistics()['sink1']?.['palletsReceived']).toBe(1);
    });
  });

  describe('elevator exits', () => {
    it('C: the pallet leaves backwards through OUT_NEAR onto the mezzanine conveyor, same side along travel', () => {
      const layout = elevatorLine('near', { maxPallets: 1 });
      const sim = new FlowSimulation(buildFlowNetwork(layout, REGISTRY), { seed: SEED });
      const headings = new Set<number>();
      for (let t = 0; t < 100; t += 0.25) {
        sim.runUntil(secondsToSimTime(t));
        for (const pallet of sim.snapshot().pallets.filter((p) => p.carrier === 'elevator1')) {
          headings.add(Math.round(pallet.heading * 1000) / 1000);
        }
      }
      expect([...headings].sort()).toEqual([0, Math.round(Math.PI * 1000) / 1000]);
      expect(sim.statistics()['sink1']?.['palletsReceived']).toBe(1);
      expect(sim.issues.map((i) => i.rule)).not.toContain('palletOrientationMismatch');
    });

    it('Z: the pallet keeps its direction through OUT_FAR', () => {
      const sim = run(elevatorLine('far', { maxPallets: 1 }), 100);
      expect(sim.statistics()['sink1']?.['palletsReceived']).toBe(1);
      expect(sim.statistics()['elevator1']?.['palletsMoved']).toBe(1);
    });

    it('warns about two connected exits and uses OUT_NEAR', () => {
      const sim = run(elevatorLine('both', { maxPallets: 1 }), 100);
      expect(sim.issues.map((i) => i.rule)).toContain('twoExits');
      expect(sim.statistics()['sink1']?.['palletsReceived']).toBe(1);
      expect(sim.statistics()['sink2']?.['palletsReceived']).toBe(0);
    });

    it('warns when no exit is connected and keeps the pallet', () => {
      const sim = run(elevatorLine('none', { maxPallets: 1 }), 100);
      expect(sim.issues.map((i) => i.rule)).toContain('noExit');
      expect(sim.statistics()['elevator1']?.['palletsMoved']).toBe(0);
    });
  });

  describe('cycle', () => {
    it('carries one pallet at a time and reports the deck height between entry and exit', () => {
      const sim = new FlowSimulation(buildFlowNetwork(scissorLine({ interval: 1 }), REGISTRY), { seed: SEED });
      const heights = new Set<number>();
      for (let t = 0; t < 300; t += 0.5) {
        sim.runUntil(secondsToSimTime(t));
        const snapshot = sim.snapshot();
        expect(snapshot.pallets.filter((p) => p.carrier === 'scissor_lift1').length).toBeLessThanOrEqual(1);
        const height = snapshot.heights['scissor_lift1'] ?? NaN;
        expect(height).toBeGreaterThanOrEqual(0.6 - 1e-9);
        expect(height).toBeLessThanOrEqual(1.2 + 1e-9);
        heights.add(Math.round(height * 100));
      }
      expect(heights.size).toBeGreaterThan(10);
    });

    it('stays at the exit height, blocked, while the next conveyor is full', () => {
      const sim = run(scissorLine({ interval: 1 }, {}, { withSink: false }), 600);
      expect(sim.snapshot().status['scissor_lift1']).toBe('blocked');
      expect(sim.snapshot().heights['scissor_lift1']).toBeCloseTo(1.2, 9);
    });

    it('splits the time into utilization, blocked and idle shares that add up to 1', () => {
      const stats = run(scissorLine({ interval: 1 }, {}, { withSink: false }), 600).statistics()['scissor_lift1'] ?? {};
      expect(stats['utilization']).toBeGreaterThan(0);
      expect(stats['blockedShare']).toBeGreaterThan(0.3);
      expect((stats['utilization'] ?? 0) + (stats['blockedShare'] ?? 0) + (stats['idleShare'] ?? 0)).toBeCloseTo(1, 9);
    });

    it('moves one pallet per cycle with a saturated source', () => {
      const sim = run(scissorLine({ interval: 1 }), 3600);
      const stats = sim.statistics();
      const cycle = stats['scissor_lift1']?.['meanCycleTime'] ?? 0;
      expect(cycle).toBeGreaterThan(T8_UP + T8_DOWN);
      expect(Math.abs((stats['sink1']?.['throughputPerHour'] ?? 0) - 3600 / cycle) / (3600 / cycle)).toBeLessThan(0.03);
    });

    it('gives the same statistics with the same seed', () => {
      const layout = elevatorLine('near', { interval: 20 });
      expect(run(layout, 1800).statistics()).toEqual(run(layout, 1800).statistics());
    });
  });

  describe('operator connections', () => {
    it('warns when the operator side is not connected', () => {
      const layout = manualOutput({ maxPallets: 1 }, undefined, { withOperator: false });
      expect(run(layout, 10).issues.map((i) => i.rule)).toContain('noOperator');
    });

    it('warns about a sink on the operator side of an input table', () => {
      const layout = manualOutput({ maxPallets: 1 }, {}, { tableParams: { flowDirection: 'in' } });
      expect(run(layout, 10).issues.map((i) => i.rule)).toContain('operatorMismatch');
    });
  });
});

type Params = Record<string, number | boolean | string>;

/** Source → roller (0.6 m) → scissor table → roller (exit height) → sink, along +x. */
function scissorLine(source: Params, table: Params = {}, { withSink = true } = {}): Layout {
  const entry = Number(table['entryHeight'] ?? 0.6);
  const exit = Number(table['exitHeight'] ?? 1.2);
  const parts: [string, Params, number][] = [
    ['flow.source', { maxPallets: 0, ...source, surfaceHeight: entry }, 0.4],
    ['conveyor.roller', { surfaceHeight: entry }, 3],
    ['conveyor.scissor_lift', table, 1.6],
    ['conveyor.roller', { surfaceHeight: exit }, 3],
  ];
  if (withSink) parts.push(['flow.sink', { surfaceHeight: exit }, 0]);
  let x = 0;
  return connect(
    parts.map(([definitionId, params, length]) => {
      const instance = place(definitionId, params, { x, y: 0, z: 0 });
      x += length + 0.02;
      return instance;
    }),
  );
}

/**
 * Source → roller (0.6 m) → elevator; the exits lead to rollers on a mezzanine at 4.0 m:
 * `near` back over the entry (C), `far` beyond the elevator (Z), `both`, or `none`.
 */
function elevatorLine(exits: 'near' | 'far' | 'both' | 'none', source: Params): Layout {
  const mezzanine = 4;
  const instances = [
    place('flow.source', { maxPallets: 0, ...source }, { x: -3.44, y: 0, z: 0 }),
    place('conveyor.roller', {}, { x: -3.02, y: 0, z: 0 }),
    place('conveyor.elevator', {}, { x: 0, y: 0, z: 0 }),
  ];
  if (exits === 'near' || exits === 'both') {
    instances.push(
      place('conveyor.roller', {}, { x: -0.02, y: mezzanine, z: 0 }, Math.PI),
      place('flow.sink', {}, { x: -3.04, y: mezzanine, z: 0 }, Math.PI),
    );
  }
  if (exits === 'far' || exits === 'both') {
    instances.push(
      place('conveyor.roller', {}, { x: 1.62, y: mezzanine, z: 0 }),
      place('flow.sink', {}, { x: 4.64, y: mezzanine, z: 0 }),
    );
  }
  return connect(instances);
}

/** Operator (source at 0.1 m) → manual table → roller (0.6 m) → sink, along +x. */
function manualInput(source: Params): Layout {
  return connect([
    place('flow.source', { maxPallets: 0, ...source, surfaceHeight: 0.1 }, { x: 0, y: 0, z: 0 }),
    place('conveyor.lift_table_manual', { flowDirection: 'in' }, { x: 0.42, y: 0, z: 0 }),
    place('conveyor.roller', {}, { x: 2.04, y: 0, z: 0 }),
    place('flow.sink', {}, { x: 5.06, y: 0, z: 0 }),
  ]);
}

/**
 * Source → roller (0.6 m) → manual table (turned 180°: LINE faces the roller) → operator (sink at
 * 0.1 m) along +x.
 */
function manualOutput(
  source: Params,
  operator: Params = {},
  { withOperator = true, tableParams = {} as Params } = {},
): Layout {
  const instances = [
    place('flow.source', { maxPallets: 0, ...source }, { x: 0, y: 0, z: 0 }),
    place('conveyor.roller', {}, { x: 0.42, y: 0, z: 0 }),
    place('conveyor.lift_table_manual', { flowDirection: 'out', ...tableParams }, { x: 5.04, y: 0, z: 0 }, Math.PI),
  ];
  if (withOperator) instances.push(place('flow.sink', { ...operator, surfaceHeight: 0.1 }, { x: 5.06, y: 0, z: 0 }));
  return connect(instances);
}

const counters = new Map<string, number>();

function place(definitionId: string, params: Params, position: Vec3, yaw = 0): Instance {
  const definition = REGISTRY.get(definitionId);
  const prefix = definitionId.split('.')[1] ?? definitionId;
  return {
    id: prefix,
    definitionId,
    definitionVersion: definition.version,
    name: '',
    position,
    yaw,
    params: instanceParams(definition, params),
    attributes: {},
  };
}

/** Numbers the instances per kind (roller1, roller2…) and connects them by their ports. */
function connect(instances: Instance[]): Layout {
  counters.clear();
  const numbered = instances.map((instance) => {
    const count = (counters.get(instance.id) ?? 0) + 1;
    counters.set(instance.id, count);
    return { ...instance, id: `${instance.id}${count}` };
  });
  const layout: Layout = { instances: numbered, connections: [] };
  return { ...layout, connections: deriveConnections(layout, REGISTRY) };
}

function run(layout: Layout, seconds: number): FlowSimulation {
  const sim = new FlowSimulation(buildFlowNetwork(layout, REGISTRY), { seed: SEED });
  sim.runUntil(secondsToSimTime(seconds));
  return sim;
}

/** Status changes of one object, event by event: [time (s), status]. */
function timeline(layout: Layout, id: string, seconds: number): [number, string][] {
  const sim = new FlowSimulation(buildFlowNetwork(layout, REGISTRY), { seed: SEED });
  const steps: [number, string][] = [];
  const record = () => {
    const status = sim.snapshot().status[id] ?? '';
    if (steps.at(-1)?.[1] !== status) steps.push([sim.now / US_PER_S, status]);
  };
  record();
  for (;;) {
    const next = sim.nextEventTime();
    if (next === undefined || next > secondsToSimTime(seconds)) break;
    sim.runUntil(next);
    record();
  }
  return steps;
}

/** How long each stretch with `status` lasted (s). */
function durations(steps: [number, string][], status: string): number[] {
  return steps.flatMap(([time, s], i) => {
    const next = steps[i + 1];
    return s === status && next !== undefined ? [next[0] - time] : [];
  });
}
