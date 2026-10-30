import { type Instance, type Layout, type RuleData, secondsToSimTime } from '@intracraft/core-model';
import { createBuiltInRegistry, deriveConnections, instanceParams } from '@intracraft/library';
import { describe, expect, it } from 'vitest';
import { FlowSimulation } from './flow-simulation';
import { buildFlowNetwork } from './network';

const REGISTRY = createBuiltInRegistry();
const STOP_AT_SENSOR: RuleData[] = [
  {
    id: 'stopAtSensor',
    when: [{ signal: 'photoeye1.detected', edge: 'rising' }],
    then: [
      { signal: 'roller2.run', value: false },
      { signal: 'roller2.run', value: true, delayMs: 10_000 },
    ],
  },
];

describe('2D controller rules', () => {
  it('a sensor stops a conveyor for 10 s: the pallet arrives exactly 10 s later', () => {
    // Braking and restarting each cost v/2a = 0.25 s; standing lasts 10 − 0.5 s: 10 s in total.
    const free = cycle(layout([]));
    const stopped = cycle(layout(STOP_AT_SENSOR));
    expect(stopped - free).toBeCloseTo(10, 4);
  });

  it('shows the switched-off conveyor as stopped and publishes the signals', () => {
    const sim = new FlowSimulation(buildFlowNetwork(layout(STOP_AT_SENSOR), REGISTRY), { seed: 1 });
    let seen = false;
    for (let t = 0; t < 40 && !seen; t += 0.1) {
      sim.runUntil(secondsToSimTime(t));
      const snapshot = sim.snapshot();
      if (snapshot.status['roller2'] !== 'stopped') continue;
      seen = true;
      expect(snapshot.signals).toMatchObject({ 'roller2.run': false, 'photoeye1.detected': true, 'roller2.occupied': true });
    }
    expect(seen).toBe(true);
  });

  it('a switched-off conveyor takes no pallet from the one before', () => {
    const rules: RuleData[] = [
      { id: 'off', when: [{ signal: 'roller1.occupied', edge: 'rising' }], then: [{ signal: 'roller2.run', value: false }] },
    ];
    const sim = new FlowSimulation(buildFlowNetwork(layout(rules), REGISTRY), { seed: 1 });
    sim.runUntil(secondsToSimTime(60));
    const [pallet] = sim.snapshot().pallets;
    expect(pallet?.carrier).toBe('roller1');
    expect(pallet?.front).toBeCloseTo(3, 6);
  });

  it('stops with an error when rules switch a signal back and forth at one instant', () => {
    const rules: RuleData[] = [
      { id: 'a', when: [{ signal: 'roller1.occupied', is: true }, { signal: 'roller2.run', is: true }], then: [{ signal: 'roller2.run', value: false }] },
      { id: 'b', when: [{ signal: 'roller2.run', is: false }], then: [{ signal: 'roller2.run', value: true }] },
    ];
    const sim = new FlowSimulation(buildFlowNetwork(layout(rules), REGISTRY), { seed: 1 });
    expect(() => sim.runUntil(secondsToSimTime(10))).toThrow(/keep changing/);
  });
});

/** Source (one pallet) → roller1 → roller2 → sink, photocell on roller2 at 1.5 m (130 mm). */
function layout(rules: RuleData[]): Layout {
  const instances = [
    instance('source1', 'flow.source', -0.42, { maxPallets: 1 }),
    instance('roller1', 'conveyor.roller', 0),
    instance('roller2', 'conveyor.roller', 3.02),
    instance('sink1', 'flow.sink', 6.04),
    instance('photoeye1', 'sensor.photoeye', 3.02, {
      host: 'roller2',
      sensorS: 1.5,
      reflectorS: 1.5,
      sensorHeight: 0.13,
      reflectorHeight: 0.13,
      responseTime: 0,
    }),
  ];
  const result: Layout = { instances, connections: [], rules };
  return { ...result, connections: deriveConnections(result, REGISTRY) };
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

function cycle(value: Layout): number {
  const sim = new FlowSimulation(buildFlowNetwork(value, REGISTRY), { seed: 1 });
  sim.runUntil(secondsToSimTime(120));
  return sim.statistics()['sink1']?.['meanCycleTime'] ?? NaN;
}
