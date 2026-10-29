// Port positions of a placed component. Local frame: x along travel, y up, z across; an instance
// is placed by its position and yaw (right-handed rotation about +Y, as three.js `rotation.y`).
import type { Instance, Vec3 } from '@intracraft/core-model';
import type { ComponentDefinition, Facing, PortCoordinate, PortDefinition } from './definition-schema';
import { type ParamValues, instanceParams } from './params';

export interface ResolvedPort {
  id: string;
  direction: PortDefinition['direction'];
  facing: Facing;
  /** Local position in metres. */
  local: Vec3;
}

export interface WorldPort {
  instanceId: string;
  portId: string;
  direction: PortDefinition['direction'];
  position: Vec3;
  /** Outward unit direction in the floor plane. */
  facing: { x: number; z: number };
}

const FACING_VECTORS: Record<Facing, { x: number; z: number }> = {
  '+x': { x: 1, z: 0 },
  '-x': { x: -1, z: 0 },
  '+z': { x: 0, z: 1 },
  '-z': { x: 0, z: -1 },
};

/** Ports of a definition with their coordinates evaluated for the given parameter values. */
export function resolvePorts(definition: ComponentDefinition, params: ParamValues): ResolvedPort[] {
  return definition.ports.map((port) => ({
    id: port.id,
    direction: port.direction,
    facing: port.facing,
    local: {
      x: evaluate(port.local.x, params, port.id),
      y: evaluate(port.local.y, params, port.id),
      z: evaluate(port.local.z, params, port.id),
    },
  }));
}

/** Ports of a placed instance in world coordinates. */
export function worldPorts(instance: Instance, definition: ComponentDefinition): WorldPort[] {
  return resolvePorts(definition, instanceParams(definition, instance.params)).map((port) =>
    placePort(port, instance.id, instance.position, instance.yaw),
  );
}

export function placePort(port: ResolvedPort, instanceId: string, position: Vec3, yaw: number): WorldPort {
  const offset = rotateY(port.local, yaw);
  return {
    instanceId,
    portId: port.id,
    direction: port.direction,
    position: { x: position.x + offset.x, y: position.y + port.local.y, z: position.z + offset.z },
    facing: rotateY(FACING_VECTORS[port.facing], yaw),
  };
}

/** Rotate a floor-plane vector by `yaw` about +Y (right-handed): +x turns toward −z. */
export function rotateY(v: { x: number; z: number }, yaw: number): { x: number; z: number } {
  const c = Math.cos(yaw);
  const s = Math.sin(yaw);
  return { x: v.x * c + v.z * s, z: -v.x * s + v.z * c };
}

function evaluate(coordinate: PortCoordinate, params: ParamValues, portId: string): number {
  if (typeof coordinate === 'number') return coordinate;
  if (typeof coordinate === 'string') return numberParam(params, coordinate, portId);
  return coordinate.reduce<number>(
    (sum, term) => sum + (typeof term === 'number' ? term : term.factor * numberParam(params, term.param, portId)),
    0,
  );
}

function numberParam(params: ParamValues, key: string, portId: string): number {
  const value = params[key];
  if (typeof value !== 'number') throw new Error(`Port ${portId}: parameter "${key}" is not a number`);
  return value;
}
