import type { Instance } from '@intracraft/core-model';
import { describe, expect, it } from 'vitest';
import { resolvePorts, rotateY, worldPorts } from './ports';
import { createBuiltInRegistry, defaultParams } from './registry';

const registry = createBuiltInRegistry();

describe('resolvePorts', () => {
  it('places the roller conveyor ports at both ends on the support plane', () => {
    const roller = registry.get('conveyor.roller');
    const [a, b] = resolvePorts(roller, defaultParams(roller));
    expect(a).toMatchObject({ id: 'A', facing: '-x', local: { x: 0, y: 0.6, z: 0 } });
    expect(b).toMatchObject({ id: 'B', facing: '+x', local: { x: 3, y: 0.6, z: 0 } });
  });

  it('puts the transfer chain ports at roller height + lift stroke, on both sides', () => {
    const transfer = registry.get('conveyor.transfer_rc');
    const ports = resolvePorts(transfer, defaultParams(transfer));
    const c1 = ports.find((p) => p.id === 'C1');
    const c2 = ports.find((p) => p.id === 'C2');
    expect(c1?.local.y).toBeCloseTo(0.64, 12);
    expect(c1?.local.x).toBeCloseTo(0.7, 12);
    expect(c1?.local.z).toBeCloseTo(-0.5, 12);
    expect(c2?.local.z).toBeCloseTo(0.5, 12);
  });

  it('gives a turntable one port per station at half the platform length', () => {
    const turntable = registry.get('conveyor.roller_turntable');
    const ports = resolvePorts(turntable, defaultParams(turntable));
    expect(ports.map((p) => p.id)).toEqual(['P0', 'P90', 'P180', 'P270']);
    expect(ports[1]).toMatchObject({ facing: '-z', local: { x: 0, z: -0.7 } });
  });
});

describe('rotateY', () => {
  it('turns +x into −z for a +90° yaw (right-handed about +Y)', () => {
    const r = rotateY({ x: 1, z: 0 }, Math.PI / 2);
    expect(r.x).toBeCloseTo(0, 12);
    expect(r.z).toBeCloseTo(-1, 12);
  });
});

describe('worldPorts', () => {
  it('moves and rotates the ports with the instance', () => {
    const conveyor = instance('roller1', 'conveyor.roller', { x: 1, y: 0, z: 2 }, Math.PI / 2);
    const [a, b] = worldPorts(conveyor, registry.get('conveyor.roller'));
    expect(a?.position.x).toBeCloseTo(1, 12);
    expect(b?.position.x).toBeCloseTo(1, 12);
    expect(b?.position.z).toBeCloseTo(-1, 12);
    expect(b?.position.y).toBeCloseTo(0.6, 12);
    expect(b?.facing.z).toBeCloseTo(-1, 12);
  });

  it('uses the instance parameters over the defaults', () => {
    const conveyor = { ...instance('roller1', 'conveyor.roller'), params: { length: 5 } };
    expect(worldPorts(conveyor, registry.get('conveyor.roller'))[1]?.position.x).toBeCloseTo(5, 12);
  });
});

function instance(id: string, definitionId: string, position = { x: 0, y: 0, z: 0 }, yaw = 0): Instance {
  return { id, definitionId, definitionVersion: '0.1.0', name: id, position, yaw, params: {}, attributes: {} };
}
