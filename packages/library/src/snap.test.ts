import type { Instance, Layout } from '@intracraft/core-model';
import { describe, expect, it } from 'vitest';
import { createBuiltInRegistry } from './registry';
import { GRID_STEP, PORT_GAP, connectionIssues, deriveConnections, snapPose } from './snap';

const registry = createBuiltInRegistry();
const roller = registry.get('conveyor.roller');

describe('snapPose', () => {
  it("puts the new conveyor's A port 20 mm in front of the existing conveyor's B port", () => {
    const layout = layoutOf(instance('roller1', 'conveyor.roller'));
    const result = snapPose({ definition: roller, params: {}, position: at(3.15, 0.1), yaw: 0 }, layout, registry);
    expect(result.link).toEqual({ movingPort: 'A', target: { instanceId: 'roller1', portId: 'B' } });
    expect(result.position.x).toBeCloseTo(3 + PORT_GAP, 12);
    expect(result.position.z).toBeCloseTo(0, 12);
    expect(result.issues).toEqual([]);
  });

  it('also snaps with the B end of the moving conveyor', () => {
    const layout = layoutOf(instance('roller1', 'conveyor.roller', at(5, 0)));
    const result = snapPose({ definition: roller, params: {}, position: at(1.9, 0), yaw: 0 }, layout, registry);
    expect(result.link).toEqual({ movingPort: 'B', target: { instanceId: 'roller1', portId: 'A' } });
    expect(result.position.x).toBeCloseTo(5 - PORT_GAP - 3, 12);
  });

  it('does not snap when the ports do not face each other', () => {
    const layout = layoutOf(instance('roller1', 'conveyor.roller'));
    const result = snapPose(
      { definition: roller, params: {}, position: at(3.1, 0), yaw: Math.PI / 2 },
      layout,
      registry,
    );
    expect(result.link).toBeNull();
  });

  it('falls back to the 100 mm grid when no port is within reach', () => {
    const layout = layoutOf(instance('roller1', 'conveyor.roller'));
    const result = snapPose({ definition: roller, params: {}, position: at(3.6, 1.234), yaw: 0 }, layout, registry);
    expect(result.link).toBeNull();
    expect(result.position.x).toBeCloseTo(3.6, 12);
    expect(result.position.z).toBeCloseTo(1.2, 12);
    expect(GRID_STEP).toBe(0.1);
  });

  it('ignores ports that are already connected', () => {
    const layout = withConnections(
      layoutOf(instance('roller1', 'conveyor.roller'), instance('roller2', 'conveyor.roller', at(3.02, 0))),
    );
    const result = snapPose({ definition: roller, params: {}, position: at(3.1, 0), yaw: 0 }, layout, registry);
    expect(result.link?.target.instanceId).not.toBe('roller1');
  });

  it('warns when the support heights differ by more than 10 mm', () => {
    const layout = layoutOf(instance('roller1', 'conveyor.roller'));
    const result = snapPose(
      { definition: roller, params: { surfaceHeight: 0.65 }, position: at(3.1, 0), yaw: 0 },
      layout,
      registry,
    );
    expect(result.link).not.toBeNull();
    expect(result.issues.map((i) => i.rule)).toEqual(['portHeightMismatch']);
  });

  it('turns the chain port of a transfer onto a chain conveyor rotated by −90°', () => {
    const layout = layoutOf(instance('transfer1', 'conveyor.transfer_rc'));
    const chain = registry.get('conveyor.chain');
    // C2 sits at (0.7, 0.64, 0.5) facing +z; a chain conveyor at yaw −90° runs along +z.
    const result = snapPose(
      { definition: chain, params: { surfaceHeight: 0.64 }, position: at(0.75, 0.6), yaw: -Math.PI / 2 },
      layout,
      registry,
    );
    expect(result.link).toEqual({ movingPort: 'A', target: { instanceId: 'transfer1', portId: 'C2' } });
    expect(result.position.x).toBeCloseTo(0.7, 12);
    expect(result.position.z).toBeCloseTo(0.5 + PORT_GAP, 12);
  });
});

describe('deriveConnections', () => {
  it('connects two conveyors placed end to end, from the out port to the in port', () => {
    const layout = layoutOf(instance('roller2', 'conveyor.roller', at(3.02, 0)), instance('roller1', 'conveyor.roller'));
    const connections = deriveConnections(layout, registry);
    expect(connections).toHaveLength(1);
    expect(connections[0]?.from).toEqual({ instanceId: 'roller1', portId: 'B' });
    expect(connections[0]?.to).toEqual({ instanceId: 'roller2', portId: 'A' });
    expect(connections[0]?.gap).toBeCloseTo(0.02, 12);
  });

  it('does not connect conveyors that are too far apart or side by side', () => {
    const apart = layoutOf(instance('roller1', 'conveyor.roller'), instance('roller2', 'conveyor.roller', at(3.2, 0)));
    const side = layoutOf(instance('roller1', 'conveyor.roller'), instance('roller2', 'conveyor.roller', at(0, 1.5)));
    expect(deriveConnections(apart, registry)).toEqual([]);
    expect(deriveConnections(side, registry)).toEqual([]);
  });

  it('connects a conveyor to a turntable station port', () => {
    // Turntable centre at x = 3.02 + 0.7: its P180 port is at x = 3.02, 20 mm after the conveyor end.
    const layout = layoutOf(
      instance('roller1', 'conveyor.roller'),
      instance('roller_turntable1', 'conveyor.roller_turntable', at(3.72, 0)),
    );
    const connections = deriveConnections(layout, registry);
    expect(connections.map((c) => `${c.from.portId}→${c.to.instanceId}.${c.to.portId}`)).toEqual([
      'B→roller_turntable1.P180',
    ]);
  });
});

describe('connectionIssues', () => {
  it('reports connected ports at different heights', () => {
    const lower = { ...instance('roller2', 'conveyor.roller', at(3.02, 0)), params: { surfaceHeight: 0.5 } };
    const layout = withConnections(layoutOf(instance('roller1', 'conveyor.roller'), lower));
    expect(connectionIssues(layout, registry).map((i) => i.rule)).toEqual(['portHeightMismatch']);
    expect(connectionIssues(layout, registry, 'roller2')).toHaveLength(1);
    expect(connectionIssues(layout, registry, 'roller9')).toEqual([]);
  });
});

function at(x: number, z: number) {
  return { x, y: 0, z };
}

function instance(id: string, definitionId: string, position = at(0, 0), yaw = 0): Instance {
  return { id, definitionId, definitionVersion: '0.1.0', name: id, position, yaw, params: {}, attributes: {} };
}

function layoutOf(...instances: Instance[]): Layout {
  return { instances, connections: [] };
}

function withConnections(layout: Layout): Layout {
  return { ...layout, connections: deriveConnections(layout, registry) };
}
