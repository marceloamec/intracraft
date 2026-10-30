import { type Instance, type Layout, secondsToSimTime } from '@intracraft/core-model';
import { createBuiltInRegistry, deriveConnections, instanceParams } from '@intracraft/library';
import { describe, expect, it } from 'vitest';
import { FlowSimulation } from './flow-simulation';
import { buildFlowNetwork } from './network';

const REGISTRY = createBuiltInRegistry();
/** Yaw that points an object's local +x to world +z (or −z). */
const TO_PLUS_Z = (3 * Math.PI) / 2;
const TO_MINUS_Z = Math.PI / 2;
/** Transfer table A at x = 3.02; 1.4 m long, 1.0 m wide → centre x = 3.72, chain ports at z = ±0.5. */
const TX = 3.02;
const CX = TX + 0.7;
const CHAIN_HEIGHT = 0.64;

describe('2D roller-to-chain transfer', () => {
  it('lifts the pallet off the rollers and sends it sideways to the chain line, short side first', () => {
    const sim = run(rollersToChains({ maxPallets: 1 }), 120);
    const stats = sim.statistics();
    expect(stats['sink1']?.['palletsReceived']).toBe(1);
    expect(stats['transfer_rc1']).toMatchObject({ transfers: 1, palletsProcessed: 1 });
    expect(sim.issues.map((i) => i.rule)).not.toContain('palletOrientationMismatch');
  });

  it('adds the lift and lowering times to its cycle (T4 structure)', () => {
    const cycle = (params: Record<string, number>) =>
      run(rollersToChains({ maxPallets: 1 }, params), 120).statistics()['transfer_rc1']?.['meanCycleTime'] ?? 0;
    expect(cycle({ liftTime: 2, lowerTime: 2.5 }) - cycle({ liftTime: 1, lowerTime: 1 })).toBeCloseTo(2.5, 5);
  });

  it('keeps the roller section closed while the chains work: one pallet per cycle', () => {
    const stats = run(rollersToChains({ interval: 1 }), 3600).statistics();
    const cycle = stats['transfer_rc1']?.['meanCycleTime'] ?? 0;
    expect(stats['transfer_rc1']?.['utilization']).toBeGreaterThan(0.9);
    expect(stats['sink1']?.['throughputPerHour']).toBeLessThanOrEqual(3600 / cycle + 1);
  });

  it('warns and keeps the pallet when the chosen side is not connected', () => {
    const sim = run(rollersToChains({ maxPallets: 1 }, { transferDirection: 'left' }), 60);
    expect(sim.issues.map((i) => i.rule)).toContain('noExit');
    expect(sim.snapshot().pallets[0]?.carrier).toBe('transfer_rc1');
  });

  it('sends pallets to both sides in turn when bidirectional', () => {
    const layout = rollersToChains({ maxPallets: 4, interval: 15 }, { transferDirection: 'bidirectional' }, true);
    const stats = run(layout, 600).statistics();
    expect([stats['sink1']?.['palletsReceived'], stats['sink2']?.['palletsReceived']]).toEqual([2, 2]);
  });

  it('takes pallets from the chains, sets them down and sends them on along the rollers', () => {
    const sim = run(chainsToRollers(), 120);
    const stats = sim.statistics();
    expect(stats['sink1']?.['palletsReceived']).toBe(1);
    expect(stats['transfer_rc1']?.['transfers']).toBe(1);
    expect(sim.issues.map((i) => i.rule)).not.toContain('palletOrientationMismatch');
  });
});

/** Source → roller1 → transfer (A) → C2 → chain1 → sink1 (and C1 → chain2 → sink2 when `both`). */
function rollersToChains(source: Record<string, number>, transfer: Record<string, number | string> = {}, both = false): Layout {
  const instances = [
    instance('source1', 'flow.source', -0.42, 0, 0, source),
    instance('roller1', 'conveyor.roller', 0, 0, 0),
    instance('transfer_rc1', 'conveyor.transfer_rc', TX, 0, 0, transfer),
    instance('chain1', 'conveyor.chain', CX, 0.52, TO_PLUS_Z, { surfaceHeight: CHAIN_HEIGHT }),
    instance('sink1', 'flow.sink', CX, 3.54, TO_PLUS_Z, { surfaceHeight: CHAIN_HEIGHT }),
  ];
  if (both) {
    instances.push(
      instance('chain2', 'conveyor.chain', CX, -0.52, TO_MINUS_Z, { surfaceHeight: CHAIN_HEIGHT }),
      instance('sink2', 'flow.sink', CX, -3.54, TO_MINUS_Z, { surfaceHeight: CHAIN_HEIGHT }),
    );
  }
  return connect(instances);
}

/** Source → chain1 (running +z) → transfer C1 → B → roller1 → sink1. */
function chainsToRollers(): Layout {
  const centre = 0.7;
  return connect([
    instance('source1', 'flow.source', centre, -3.94, TO_PLUS_Z, { maxPallets: 1, orientation: 'short', surfaceHeight: CHAIN_HEIGHT }),
    instance('chain1', 'conveyor.chain', centre, -3.52, TO_PLUS_Z, { surfaceHeight: CHAIN_HEIGHT }),
    instance('transfer_rc1', 'conveyor.transfer_rc', 0, 0, 0, { operationMode: 'chainsToRollers' }),
    instance('roller1', 'conveyor.roller', 1.42, 0, 0),
    instance('sink1', 'flow.sink', 4.44, 0, 0),
  ]);
}

function connect(instances: Instance[]): Layout {
  const layout: Layout = { instances, connections: [] };
  return { ...layout, connections: deriveConnections(layout, REGISTRY) };
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

function run(layout: Layout, seconds: number): FlowSimulation {
  const sim = new FlowSimulation(buildFlowNetwork(layout, REGISTRY), { seed: 5 });
  sim.runUntil(secondsToSimTime(seconds));
  return sim;
}
