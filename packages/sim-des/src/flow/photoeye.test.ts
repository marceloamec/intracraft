import { type Instance, type Layout, secondsToSimTime } from '@intracraft/core-model';
import { createBuiltInRegistry, deriveConnections, instanceParams } from '@intracraft/library';
import { describe, expect, it } from 'vitest';
import { FlowSimulation } from './flow-simulation';
import { buildFlowNetwork } from './network';

const REGISTRY = createBuiltInRegistry();
/** EUR long side (1.2 m) at 0.25 m/s. */
const PASS_TIME = 4.8;
/** Top deck of an EUR pallet on rollers: a continuous band (S06 §2). */
const CONTINUOUS = 0.13;

describe('2D photoelectric sensor', () => {
  it('T6: beam at 2.0 m in a continuous band → one pulse of 4.8 s per pallet', () => {
    const stats = statsOf(sensor({ sensorS: 2, reflectorS: 2 }));
    expect(stats).toMatchObject({ pulses: 1 });
    expect(stats['occupiedTime']).toBeCloseTo(PASS_TIME, 5);
  });

  it('flickers three times per pallet at 60 mm on rollers; a 2 s filter makes it one pulse', () => {
    expect(statsOf(sensor({ sensorS: 2, reflectorS: 2, sensorHeight: 0.06, reflectorHeight: 0.06 }))['pulses']).toBe(3);
    const filtered = statsOf(sensor({ sensorS: 2, reflectorS: 2, sensorHeight: 0.06, reflectorHeight: 0.06, filter: 2 }));
    expect(filtered['pulses']).toBe(1);
    // The filter is an off-delay: the output stays on 2 s after the tail has passed.
    expect(filtered['occupiedTime']).toBeCloseTo(PASS_TIME + 2, 5);
  });

  it('an oblique beam covers more of the conveyor: occupied while any part of it is blocked', () => {
    // Beam from (1.8 m, −0.8 m) to (2.2 m, +0.8 m) crosses the 0.8 m wide pallet between 1.9 and 2.1 m.
    const stats = statsOf(sensor({ sensorS: 1.8, reflectorS: 2.2 }));
    expect(stats['occupiedTime']).toBeCloseTo((1.2 + 0.2) / 0.25, 5);
  });

  it('sees the load above the pallet, and nothing above the load', () => {
    expect(statsOf(sensor({ sensorS: 2, reflectorS: 2, sensorHeight: 0.5, reflectorHeight: 0.5 }))['occupiedTime']).toBeCloseTo(PASS_TIME, 5);
    expect(statsOf(sensor({ sensorS: 2, reflectorS: 2, sensorHeight: 1.5, reflectorHeight: 1.5 }))['pulses']).toBe(0);
  });

  it('detects a pallet still crossing from the previous conveyor', () => {
    const stats = statsOf(sensor({ host: 'roller2', sensorS: 0.1, reflectorS: 0.1 }));
    expect(stats['pulses']).toBe(1);
    expect(stats['occupiedTime']).toBeCloseTo(PASS_TIME, 5);
  });

  it('delays the output by the response time without changing the pulse length', () => {
    const sim = run(sensor({ sensorS: 2, reflectorS: 2, responseTime: 0.05 }), 0);
    const risesAt = (s: FlowSimulation) => {
      for (let t = 0; t < 30; t += 0.001) {
        s.runUntil(secondsToSimTime(t));
        if (s.snapshot().status['photoeye1'] === 'detecting') return t;
      }
      return NaN;
    };
    const delayed = risesAt(sim);
    const immediate = risesAt(run(sensor({ sensorS: 2, reflectorS: 2, responseTime: 0 }), 0));
    expect(delayed - immediate).toBeCloseTo(0.05, 2);
  });

  it('warns about a missing host, voids at 60 mm and an inclined beam', () => {
    const rules = (params: Record<string, number | string>) => run(sensor(params), 0).issues.map((i) => i.rule);
    expect(rules({ host: '' })).toContain('sensorWithoutHost');
    expect(rules({ sensorHeight: 0.06, reflectorHeight: 0.06 })).toContain('beamInVoidBand');
    expect(rules({ sensorHeight: 0.06, reflectorHeight: 0.13 })).toContain('inclinedBeam');
    expect(rules({ sensorHeight: CONTINUOUS, reflectorHeight: CONTINUOUS })).toEqual([]);
  });
});

/** Source (one pallet) → roller1 → roller2 → sink, with a photocell (default on roller1, 130 mm). */
function sensor(params: Record<string, number | string>): Layout {
  const instances = [
    instance('source1', 'flow.source', -0.42, { maxPallets: 1 }),
    instance('roller1', 'conveyor.roller', 0),
    instance('roller2', 'conveyor.roller', 3.02),
    instance('sink1', 'flow.sink', 6.04),
    instance('photoeye1', 'sensor.photoeye', 0, {
      host: 'roller1',
      sensorHeight: CONTINUOUS,
      reflectorHeight: CONTINUOUS,
      responseTime: 0,
      ...params,
    }),
  ];
  const layout: Layout = { instances, connections: [] };
  return { ...layout, connections: deriveConnections(layout, REGISTRY) };
}

function instance(id: string, definitionId: string, x: number, params: Record<string, number | string> = {}): Instance {
  const definition = REGISTRY.get(definitionId);
  return {
    id,
    definitionId,
    definitionVersion: definition.version,
    name: id,
    position: { x, y: 0, z: 0 },
    yaw: 0,
    params: instanceParams(definition, params),
    attributes: {},
  };
}

function run(layout: Layout, seconds: number): FlowSimulation {
  const sim = new FlowSimulation(buildFlowNetwork(layout, REGISTRY), { seed: 1 });
  sim.runUntil(secondsToSimTime(seconds));
  return sim;
}

function statsOf(layout: Layout): Record<string, number> {
  return run(layout, 120).statistics()['photoeye1'] ?? {};
}
