// Mounting a photocell on a conveyor: the conveyor under the pointer becomes its host and the
// pointer's distance along it the sensor position (S06: positions are in the host frame).
import type { Instance, Layout, Vec3 } from '@intracraft/core-model';
import { type DefinitionRegistry, instanceParams, num, rotateY } from '@intracraft/library';

/** Behaviours that can carry a sensor: straight conveyors. */
const HOST_BEHAVIORS = new Set(['des.roller_conveyor', 'des.chain_conveyor']);
/** How far beside the conveyor (beyond half its width) the pointer may be (m). */
const SIDE_MARGIN = 0.3;
/** Sensor positions snap to this step along the conveyor (m). */
const STEP = 0.01;

export interface Mount {
  host: Instance;
  /** Distance from the host entry (m). */
  s: number;
}

export function isSensor(definitionId: string, registry: DefinitionRegistry): boolean {
  return registry.get(definitionId).behaviors.des === 'des.photoeye';
}

/** The straight conveyor under `pointer` and the position along it, or undefined. */
export function findMount(layout: Layout, registry: DefinitionRegistry, pointer: Vec3): Mount | undefined {
  for (const host of layout.instances) {
    const definition = registry.get(host.definitionId);
    if (!HOST_BEHAVIORS.has(definition.behaviors.des)) continue;
    const params = instanceParams(definition, host.params);
    const local = rotateY({ x: pointer.x - host.position.x, z: pointer.z - host.position.z }, -host.yaw);
    const length = num(params, 'length', 0);
    if (local.x < 0 || local.x > length || Math.abs(local.z) > num(params, 'usableWidth', 0) / 2 + SIDE_MARGIN) continue;
    return { host, s: Math.round(local.x / STEP) * STEP };
  }
  return undefined;
}

/** Instance pose and parameters of a sensor mounted at `mount` (both beam ends at the same position). */
export function mountedSensor(mount: Mount): { position: Vec3; yaw: number; params: Record<string, string | number> } {
  return {
    position: mount.host.position,
    yaw: mount.host.yaw,
    params: { host: mount.host.id, sensorS: mount.s, reflectorS: mount.s },
  };
}

/** Where to draw an instance: a mounted sensor follows its host, everything else its own pose. */
export function displayPose(instance: Instance, layout: Layout): { position: Vec3; yaw: number } {
  const hostId = instance.params['host'];
  const host = typeof hostId === 'string' && hostId !== '' ? layout.instances.find((i) => i.id === hostId) : undefined;
  return host === undefined ? instance : host;
}
