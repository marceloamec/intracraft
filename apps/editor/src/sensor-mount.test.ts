import type { Instance, Layout } from '@intracraft/core-model';
import { createBuiltInRegistry } from '@intracraft/library';
import { describe, expect, it } from 'vitest';
import { findMount, isSensor, mountedSensor } from './sensor-mount';

const REGISTRY = createBuiltInRegistry();

describe('mounting a photocell on a conveyor', () => {
  it('finds the conveyor under the pointer and the distance along it', () => {
    const mount = findMount(layoutWith(roller(0, 0, 0)), REGISTRY, { x: 1.234, y: 0, z: 0.2 });
    expect(mount?.host.id).toBe('roller1');
    expect(mount?.s).toBeCloseTo(1.23, 9);
  });

  it('measures along a rotated conveyor', () => {
    // A quarter turn points the conveyor's local +x to world −z.
    const mount = findMount(layoutWith(roller(5, 2, Math.PI / 2)), REGISTRY, { x: 5.1, y: 0, z: 0.5 });
    expect(mount?.s).toBeCloseTo(1.5, 9);
  });

  it('ignores pointers beside or beyond the conveyor', () => {
    const layout = layoutWith(roller(0, 0, 0));
    expect(findMount(layout, REGISTRY, { x: 1, y: 0, z: 1.5 })).toBeUndefined();
    expect(findMount(layout, REGISTRY, { x: 3.5, y: 0, z: 0 })).toBeUndefined();
  });

  it('places the sensor on the host with both beam ends at the same position', () => {
    const host = roller(1, 2, 0);
    expect(mountedSensor({ host, s: 0.8 })).toEqual({
      position: host.position,
      yaw: 0,
      params: { host: 'roller1', sensorS: 0.8, reflectorS: 0.8 },
    });
    expect(isSensor('sensor.photoeye', REGISTRY)).toBe(true);
    expect(isSensor('conveyor.roller', REGISTRY)).toBe(false);
  });
});

function roller(x: number, z: number, yaw: number): Instance {
  return {
    id: 'roller1',
    definitionId: 'conveyor.roller',
    definitionVersion: '0.1.0',
    name: 'roller1',
    position: { x, y: 0, z },
    yaw,
    params: {},
    attributes: {},
  };
}

function layoutWith(...instances: Instance[]): Layout {
  return { instances, connections: [] };
}
