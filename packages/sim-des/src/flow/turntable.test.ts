import { type Instance, type Layout, secondsToSimTime } from '@intracraft/core-model';
import { createBuiltInRegistry, deriveConnections, instanceParams } from '@intracraft/library';
import { describe, expect, it } from 'vitest';
import { FlowSimulation } from './flow-simulation';
import { buildFlowNetwork } from './network';

const REGISTRY = createBuiltInRegistry();
const QUARTER = Math.PI / 2;
/** T3 (6.5 s for 90°) plus the default settle time of 0.3 s. */
const TURN_90 = 6.8;
/** Turntable centre: roller1 ends at x = 3, the platform (1.4 m) starts 20 mm later. */
const CX = 3.72;

describe('2D turntable', () => {
  it('passes a pallet straight through without turning', () => {
    const sim = run(layout('straight', { maxPallets: 1 }), 120);
    const stats = sim.statistics();
    expect(stats['sink1']?.['palletsReceived']).toBe(1);
    expect(stats['roller_turntable1']).toMatchObject({ turns: 0, palletsProcessed: 1 });
  });

  it('a 90° corner adds exactly one turn (T3 + settle) to the time on the platform', () => {
    const straight = run(layout('straight', { maxPallets: 1 }), 120).statistics()['roller_turntable1']?.['meanCycleTime'] ?? 0;
    const corner = run(layout('corner', { maxPallets: 1 }), 120).statistics();
    expect(corner['sink1']?.['palletsReceived']).toBe(1);
    expect(corner['roller_turntable1']?.['turns']).toBe(1);
    expect((corner['roller_turntable1']?.['meanCycleTime'] ?? 0) - straight).toBeCloseTo(TURN_90, 5);
  });

  it('turns back to the entry for the next pallet', () => {
    const stats = run(layout('corner', { maxPallets: 2, interval: 5 }), 300).statistics();
    expect(stats['sink1']?.['palletsReceived']).toBe(2);
    expect(stats['roller_turntable1']?.['turns']).toBe(3);
  });

  it('carries the pallet round with the platform while it turns', () => {
    const sim = new FlowSimulation(buildFlowNetwork(layout('corner', { maxPallets: 1 }), REGISTRY), { seed: 1 });
    let turningAt: number | undefined;
    for (let t = 0; t < 60 && turningAt === undefined; t += 0.05) {
      sim.runUntil(secondsToSimTime(t));
      if (sim.snapshot().status['roller_turntable1'] === 'turning') turningAt = t;
    }
    if (turningAt === undefined) throw new Error('never turned');
    sim.runUntil(secondsToSimTime(turningAt + 6.5 / 2));
    const { pallets, angles } = sim.snapshot();
    expect(pallets[0]?.carrier).toBe('roller_turntable1');
    expect(Math.abs(pallets[0]?.heading ?? 0)).toBeCloseTo(QUARTER / 2, 1);
    expect(angles['roller_turntable1']).toBe(pallets[0]?.heading);
    // The pallet stays centred on the platform: path start + half a platform along the heading.
    const p = pallets[0];
    if (p === undefined) throw new Error('no pallet');
    const along = p.front - p.length / 2;
    expect(p.pathX + along * Math.cos(p.heading)).toBeCloseTo(0, 6);
    expect(p.pathZ - along * Math.sin(p.heading)).toBeCloseTo(0, 6);
  });

  it('alternates between two exits, or always takes the first one', () => {
    const alternate = run(layout('fork', { maxPallets: 6, interval: 10 }), 900).statistics();
    expect([alternate['sink1']?.['palletsReceived'], alternate['sink2']?.['palletsReceived']]).toEqual([3, 3]);
    const first = run(layout('fork', { maxPallets: 6, interval: 10 }, 'firstExit'), 900).statistics();
    expect((first['sink1']?.['palletsReceived'] ?? 0) + (first['sink2']?.['palletsReceived'] ?? 0)).toBe(6);
    expect([first['sink1']?.['palletsReceived'], first['sink2']?.['palletsReceived']]).toContain(0);
  });

  it('chooses randomly but reproducibly with the random rule', () => {
    const counts = () => {
      const stats = run(layout('fork', { maxPallets: 20, interval: 10 }, 'random'), 3600).statistics();
      return [stats['sink1']?.['palletsReceived'] ?? 0, stats['sink2']?.['palletsReceived'] ?? 0];
    };
    const first = counts();
    expect((first[0] ?? 0) + (first[1] ?? 0)).toBe(20);
    expect(first).not.toContain(0);
    expect(counts()).toEqual(first);
  });

  it('warns when no station leads anywhere', () => {
    const noExit = layout('straight', { maxPallets: 1 });
    noExit.instances = noExit.instances.filter((i) => i.id === 'source1' || i.id === 'roller1' || i.id === 'roller_turntable1');
    const sim = run({ ...noExit, connections: deriveConnections(noExit, REGISTRY) }, 60);
    expect(sim.issues.map((i) => i.rule)).toContain('noExit');
  });
});

type Shape = 'straight' | 'corner' | 'fork';

/**
 * Source → roller1 → roller turntable, then:
 * straight: P0 → roller2 → sink1; corner: P90 → roller2 → sink1; fork: P0 → sink1 and P90 → roller2 → sink2.
 */
function layout(shape: Shape, source: Record<string, number>, exitRule = 'alternate'): Layout {
  const instances: Instance[] = [
    instance('source1', 'flow.source', -0.42, 0, 0, source),
    instance('roller1', 'conveyor.roller', 0, 0, 0),
    instance('roller_turntable1', 'conveyor.roller_turntable', CX, 0, 0, { exitRule }),
  ];
  const ahead = CX + 0.72; // P0 at CX + 0.7, plus the 20 mm gap
  const side = -0.72; // P90 at z = −0.7
  if (shape === 'straight') {
    instances.push(instance('roller2', 'conveyor.roller', ahead, 0, 0), instance('sink1', 'flow.sink', ahead + 3.02, 0, 0));
  } else {
    // Rotated a quarter turn, local +x points to world −z.
    instances.push(
      instance('roller2', 'conveyor.roller', CX, side, QUARTER),
      instance(shape === 'fork' ? 'sink2' : 'sink1', 'flow.sink', CX, side - 3.02, QUARTER),
    );
    if (shape === 'fork') instances.push(instance('sink1', 'flow.sink', ahead, 0, 0));
  }
  const result: Layout = { instances, connections: [] };
  return { ...result, connections: deriveConnections(result, REGISTRY) };
}

function instance(
  id: string,
  definitionId: string,
  x: number,
  z: number,
  yaw: number,
  params: Record<string, number | string> = {},
): Instance {
  const definition = REGISTRY.get(definitionId);
  return {
    id,
    definitionId,
    definitionVersion: definition.version,
    name: id,
    position: { x, y: 0, z },
    yaw,
    params: instanceParams(definition, params),
    attributes: {},
  };
}

function run(value: Layout, seconds: number): FlowSimulation {
  const sim = new FlowSimulation(buildFlowNetwork(value, REGISTRY), { seed: 3 });
  sim.runUntil(secondsToSimTime(seconds));
  return sim;
}
