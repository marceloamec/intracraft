import { type Instance, type Layout, secondsToSimTime } from '@intracraft/core-model';
import { createBuiltInRegistry, deriveConnections, instanceParams, num } from '@intracraft/library';
import { describe, expect, it } from 'vitest';
import { FlowSimulation } from './flow-simulation';
import { buildFlowNetwork } from './network';

const REGISTRY = createBuiltInRegistry();
const SEED = 7;

describe('2D process station (S11)', () => {
  it('T6: the processing time adds exactly to the cycle (30 s instead of 10 s → +20 s)', () => {
    const short = station({ processingTime: 10 }, 120);
    const long = station({ processingTime: 30 }, 120);
    expect(long['palletsProcessed']).toBe(1);
    expect(long['meanProcessingTime']).toBeCloseTo(30, 9);
    expect((long['meanCycleTime'] ?? 0) - (short['meanCycleTime'] ?? 0)).toBeCloseTo(20, 6);
  });

  it('T7: with a saturated source the line delivers one pallet per cycle', () => {
    const sim = run(chain({ processingTime: 30 }), 3600);
    const cycle = sim.statistics()['process_station1']?.['meanCycleTime'] ?? 0;
    const perHour = sim.statistics()['sink1']?.['throughputPerHour'] ?? 0;
    expect(Math.abs(perHour - 3600 / cycle) / (3600 / cycle)).toBeLessThan(0.03);
  });

  it('counts the processing time as utilization, not as blocked (one 30 s pallet in 120 s → 25 %)', () => {
    const stats = station({ processingTime: 30 }, 120);
    expect(stats['utilization']).toBeCloseTo(0.25, 9);
    expect(stats['blockedShare']).toBe(0);
  });

  it('splits the time into processing, blocked and idle shares that add up to 1', () => {
    const stats = run(chain({ processingTime: 5 }, { withSink: false }), 600).statistics()['process_station1'] ?? {};
    expect(stats['blockedShare']).toBeGreaterThan(0.3);
    expect(stats['utilization']).toBeGreaterThan(0);
    expect((stats['utilization'] ?? 0) + (stats['blockedShare'] ?? 0) + (stats['idleShare'] ?? 0)).toBeCloseTo(1, 9);
  });

  it('keeps one pallet at a time and reports "processing" while it works', () => {
    const sim = new FlowSimulation(buildFlowNetwork(chain({ processingTime: 30 }), REGISTRY), { seed: SEED });
    const seen = new Set<string | undefined>();
    for (let t = 0; t < 200; t += 0.5) {
      sim.runUntil(secondsToSimTime(t));
      seen.add(sim.snapshot().status['process_station1']);
      const onDeck = sim.snapshot().pallets.filter((p) => p.carrier.startsWith('process_station1'));
      expect(onDeck.length).toBeLessThanOrEqual(1);
    }
    expect(seen.has('processing')).toBe(true);
  });

  it.each(['normal', 'exponential'] as const)('draws %s times with the right mean, repeatably', (processingMode) => {
    const layout = chain({ processingMode, processingTime: 10, processingStdDev: 2 }, { interval: 1, length: 3 });
    const first = run(layout, 40000).statistics()['process_station1'] ?? {};
    expect(first['palletsProcessed']).toBeGreaterThan(1500);
    expect(Math.abs((first['meanProcessingTime'] ?? 0) - 10) / 10).toBeLessThan(0.05);
    expect(run(layout, 40000).statistics()['process_station1']).toEqual(first);
  });

  it('warns when the output is not connected and keeps the pallet', () => {
    const sim = run(chain({ processingTime: 5 }, { withSink: false, withExitConveyor: false }), 200);
    expect(sim.issues.map((i) => i.rule)).toContain('noExit');
  });

  it('warns when the deck is shorter than a pallet plus margin', () => {
    const sim = run(chain({ length: 1.2 }), 10);
    expect(sim.issues.map((i) => i.rule)).toContain('deckTooShort');
  });
});

type Params = Record<string, number | boolean | string>;
type Part = [definitionId: string, params?: Params];

/** Source → conveyor → station → conveyor → sink, in a row along +x. */
function chain(
  stationParams: Params,
  { interval = 1, length = 3, withSink = true, withExitConveyor = true } = {},
): Layout {
  const parts: Part[] = [
    ['flow.source', { interval, maxPallets: 0 }],
    ['conveyor.roller', { length }],
    ['flow.process_station', stationParams],
  ];
  if (withExitConveyor) parts.push(['conveyor.roller', { length }]);
  if (withSink && withExitConveyor) parts.push(['flow.sink']);
  return line(parts);
}

/** Statistics of the station after one pallet has gone through it. */
function station(params: Params, seconds: number): Record<string, number> {
  const layout = line([
    ['flow.source', { maxPallets: 1 }],
    ['conveyor.roller'],
    ['flow.process_station', params],
    ['conveyor.roller'],
    ['flow.sink'],
  ]);
  return run(layout, seconds).statistics()['process_station1'] ?? {};
}

/** Objects placed one after another along +x, 20 mm apart, connected port to port. */
function line(parts: Part[]): Layout {
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
