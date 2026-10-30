// Port snapping and geometric connections. Two ports are connected when they face each other and
// sit PORT_GAP apart; there is no separate "connect" step, so placing objects is enough (ADR-0005).
import type { Connection, Layout, Vec3 } from '@intracraft/core-model';
import type { ComponentDefinition } from './definition-schema';
import { type ParamValues, instanceParams } from './params';
import { type WorldPort, placePort, resolvePorts, worldPorts } from './ports';
import type { DefinitionRegistry } from './registry';
import type { ValidationIssue } from './validation-rules';

/** Gap between two connected ports (m); same default as the connection schema. */
export const PORT_GAP = 0.02;
/** Maximum distance (m) at which a dragged port snaps onto a free port. */
export const SNAP_RADIUS = 0.3;
/** Placement grid when no port is within reach (m), editor rule A09. */
export const GRID_STEP = 0.1;
/** Allowed mismatch when deciding that two ports touch (m). */
export const CONNECT_TOLERANCE = 0.01;
/** Allowed difference of support heights between connected ports (m), A04. */
export const HEIGHT_TOLERANCE = 0.01;

const OPPOSITE = -0.999; // cosine of the angle between two facing ports

export interface SnapCandidate {
  definition: ComponentDefinition;
  /** Instance parameter values (defaults are filled in). */
  params: ParamValues;
  position: Vec3;
  yaw: number;
  /** Instance being moved, ignored as a snap target. */
  instanceId?: string;
}

export interface SnapLink {
  movingPort: string;
  target: { instanceId: string; portId: string };
}

export interface SnapResult {
  position: Vec3;
  yaw: number;
  link: SnapLink | null;
  issues: ValidationIssue[];
}

/**
 * Where a component dropped or dragged at `candidate` ends up: aligned with the nearest free port
 * that faces one of its ports (within SNAP_RADIUS), otherwise on the 100 mm grid. The yaw is kept;
 * the user rotates with R.
 */
export function snapPose(
  candidate: SnapCandidate,
  layout: Layout,
  registry: DefinitionRegistry,
  radius = SNAP_RADIUS,
): SnapResult {
  const params = instanceParams(candidate.definition, candidate.params);
  const moving = resolvePorts(candidate.definition, params).map((port) =>
    placePort(port, candidate.instanceId ?? '', candidate.position, candidate.yaw),
  );
  const targets = freePorts(layout, registry, candidate.instanceId);

  let best: SnapOption | null = null;
  for (const m of moving) {
    for (const t of targets) {
      if (!facesEachOther(m, t)) continue;
      const dx = t.position.x + t.facing.x * PORT_GAP - m.position.x;
      const dz = t.position.z + t.facing.z * PORT_GAP - m.position.z;
      const option = { distance: Math.hypot(dx, dz), moving: m, target: t, dx, dz };
      if (option.distance <= radius && (best === null || better(option, best))) best = option;
    }
  }

  if (best === null) {
    return {
      position: { x: toGrid(candidate.position.x), y: candidate.position.y, z: toGrid(candidate.position.z) },
      yaw: candidate.yaw,
      link: null,
      issues: [],
    };
  }
  return {
    position: { x: candidate.position.x + best.dx, y: candidate.position.y, z: candidate.position.z + best.dz },
    yaw: candidate.yaw,
    link: { movingPort: best.moving.portId, target: { instanceId: best.target.instanceId, portId: best.target.portId } },
    issues: pairIssues(best.moving, best.target),
  };
}

/**
 * Connections implied by the geometry: facing ports PORT_GAP apart, from an out port to an in port.
 * When a port has several candidates at one position seen from above (the C elevator, ADR-0013 §7),
 * the one with the closest height wins.
 */
export function deriveConnections(layout: Layout, registry: DefinitionRegistry): Connection[] {
  const ports = allPorts(layout, registry).sort((a, b) => portKey(a).localeCompare(portKey(b)));
  const candidates: { a: WorldPort; b: WorldPort; order: number; dy: number }[] = [];
  for (const [i, a] of ports.entries()) {
    for (const [j, b] of ports.entries()) {
      if (j <= i || a.instanceId === b.instanceId) continue;
      if (!facesEachOther(a, b) || !directionsCompatible(a, b)) continue;
      const dx = a.position.x + a.facing.x * PORT_GAP - b.position.x;
      const dz = a.position.z + a.facing.z * PORT_GAP - b.position.z;
      if (Math.hypot(dx, dz) > CONNECT_TOLERANCE) continue;
      candidates.push({ a, b, order: candidates.length, dy: Math.abs(a.position.y - b.position.y) });
    }
  }
  const used = new Set<string>();
  const chosen = candidates
    .sort((p, q) => p.dy - q.dy || p.order - q.order)
    .filter(({ a, b }) => {
      if (used.has(portKey(a)) || used.has(portKey(b))) return false;
      used.add(portKey(a));
      used.add(portKey(b));
      return true;
    })
    .sort((p, q) => p.order - q.order);
  return chosen.map(({ a, b }) => {
    const [from, to] = a.direction === 'in' || b.direction === 'out' ? [b, a] : [a, b];
    return {
      from: { instanceId: from.instanceId, portId: from.portId },
      to: { instanceId: to.instanceId, portId: to.portId },
      gap: Math.hypot(a.position.x - b.position.x, a.position.z - b.position.z),
    };
  });
}

/** Warnings about existing connections (support heights that do not match), optionally of one instance. */
export function connectionIssues(layout: Layout, registry: DefinitionRegistry, instanceId?: string): ValidationIssue[] {
  const byKey = new Map(allPorts(layout, registry).map((p) => [portKey(p), p]));
  const involved = (c: Connection) =>
    instanceId === undefined || c.from.instanceId === instanceId || c.to.instanceId === instanceId;
  return layout.connections.filter(involved).flatMap((c) => {
    const from = byKey.get(`${c.from.instanceId}.${c.from.portId}`);
    const to = byKey.get(`${c.to.instanceId}.${c.to.portId}`);
    return from === undefined || to === undefined ? [] : pairIssues(from, to);
  });
}

function pairIssues(a: WorldPort, b: WorldPort): ValidationIssue[] {
  const issues: ValidationIssue[] = [];
  const dy = Math.abs(a.position.y - b.position.y);
  if (dy > HEIGHT_TOLERANCE) {
    issues.push({
      rule: 'portHeightMismatch',
      severity: 'warning',
      message: `Support heights of ${label(a)} and ${label(b)} differ by ${(dy * 1000).toFixed(0)} mm`,
    });
  }
  if (!directionsCompatible(a, b)) {
    issues.push({
      rule: 'portDirectionMismatch',
      severity: 'warning',
      message: `${label(a)} and ${label(b)} are both ${a.direction} ports`,
    });
  }
  return issues;
}

interface SnapOption {
  distance: number;
  moving: WorldPort;
  target: WorldPort;
  dx: number;
  dz: number;
}

/**
 * The nearer option wins. At practically the same distance (two ports at one position seen from
 * above) a pair whose directions fit wins, then the one with the closest height.
 */
function better(option: SnapOption, best: SnapOption): boolean {
  if (Math.abs(option.distance - best.distance) > CONNECT_TOLERANCE) return option.distance < best.distance;
  const fits = (o: SnapOption) => directionsCompatible(o.moving, o.target);
  if (fits(option) !== fits(best)) return fits(option);
  const dy = (o: SnapOption) => Math.abs(o.moving.position.y - o.target.position.y);
  if (dy(option) !== dy(best)) return dy(option) < dy(best);
  return option.distance < best.distance;
}

function freePorts(layout: Layout, registry: DefinitionRegistry, movingId: string | undefined): WorldPort[] {
  const busy = new Set<string>();
  for (const c of layout.connections) {
    if (c.from.instanceId === movingId || c.to.instanceId === movingId) continue;
    busy.add(`${c.from.instanceId}.${c.from.portId}`);
    busy.add(`${c.to.instanceId}.${c.to.portId}`);
  }
  return allPorts(layout, registry).filter((p) => p.instanceId !== movingId && !busy.has(portKey(p)));
}

function allPorts(layout: Layout, registry: DefinitionRegistry): WorldPort[] {
  return layout.instances.flatMap((instance) => worldPorts(instance, registry.get(instance.definitionId)));
}

function facesEachOther(a: WorldPort, b: WorldPort): boolean {
  return a.facing.x * b.facing.x + a.facing.z * b.facing.z < OPPOSITE;
}

function directionsCompatible(a: WorldPort, b: WorldPort): boolean {
  return !(a.direction === b.direction && a.direction !== 'inout');
}

function toGrid(value: number): number {
  return Math.round(value / GRID_STEP) * GRID_STEP;
}

function portKey(p: WorldPort): string {
  return `${p.instanceId}.${p.portId}`;
}

function label(p: WorldPort): string {
  return p.instanceId === '' ? p.portId : portKey(p);
}
