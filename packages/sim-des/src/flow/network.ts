// The plant as the 2D mode sees it: one node per instance with the values its behaviour needs and
// the instance connected to its output port. Built from the layout and the library definitions.
import type { Layout, RuleData, Vec3 } from '@intracraft/core-model';
import {
  type ComponentDefinition,
  type DefinitionRegistry,
  PALLET_TYPES,
  type ParamValues,
  type PalletType,
  instanceParams,
  num,
  resolvePorts,
} from '@intracraft/library';

/** `behaviors.des` of definitions the 2D model ignores (scenery objects, S17). */
export const NO_2D_BEHAVIOR = 'des.none';

/** Behaviours of the 2D mode, keyed by the definition's `behaviors.des`. */
const LINE_BEHAVIORS = new Set(['des.roller_conveyor', 'des.chain_conveyor']);
/** Turntable station ports and their angles (rad, right-hand about +Y; 0 = local +x). */
const STATION_PORTS: Record<string, number> = {
  P0: 0,
  P90: Math.PI / 2,
  P180: Math.PI,
  P270: (3 * Math.PI) / 2,
};

export interface FlowIssue {
  instanceId: string;
  rule: string;
  message: string;
}

interface NodeBase {
  id: string;
  name: string;
  position: Vec3;
  yaw: number;
  surfaceHeight: number;
  /** Instance connected to this node's output port. */
  downstream?: string;
  /** Port of the downstream instance that receives the pallets. */
  downstreamPort?: string;
  /** Measured gap to the downstream port (m). */
  downstreamGap: number;
}

/** Straight conveyor (rollers or chains) without accumulation: all its pallets move together. */
export interface LineNode extends NodeBase {
  kind: 'line';
  length: number;
  nominalSpeed: number;
  accel: number;
  decel: number;
  minGap: number;
  palletSideAlongTravel: 'long' | 'short';
}

export interface SourceNode extends NodeBase {
  kind: 'source';
  palletType: PalletType;
  orientation: 'long' | 'short';
  loadMass: number;
  loadHeight: number;
  generationMode: 'fixedInterval' | 'exponential';
  interval: number;
  maxPallets: number;
  blockIfOccupied: boolean;
}

export interface SinkNode extends NodeBase {
  kind: 'sink';
  removalTime: number;
}

/** A station port of a turntable connected to the entry of another object. */
export interface TurntableExit {
  portId: string;
  /** Station angle (rad): the direction a pallet leaves in, in the turntable's frame. */
  angle: number;
  target: string;
  gap: number;
}

/** Chain or roller turntable (S03, S04): receives, centres, turns to an exit station, discharges. */
export interface TurntableNode extends NodeBase {
  kind: 'turntable';
  platformLength: number;
  /** Conveying speed and ramp of the deck (chains or rollers). */
  nominalSpeed: number;
  accel: number;
  maxAngularSpeed: number;
  angularAccel: number;
  angularDecel: number;
  settleTime: number;
  /** Station angles (rad) where the platform may stop. */
  stations: number[];
  palletSideAlongTravel: 'long' | 'short';
  exitRule: 'alternate' | 'firstExit' | 'random';
  exits: TurntableExit[];
}

/**
 * Roller-to-chain transfer table (S05): rollers along x (A → B), liftable chains across
 * (C1 at −z, C2 at +z) that raise the pallet and move it sideways without turning it.
 */
export interface TransferNode extends NodeBase {
  kind: 'transfer';
  length: number;
  usableWidth: number;
  nominalSpeed: number;
  accel: number;
  transferSpeed: number;
  transferAccel: number;
  liftTime: number;
  lowerTime: number;
  liftStroke: number;
  transferDirection: 'left' | 'right' | 'bidirectional';
  operationMode: 'rollersToChains' | 'chainsToRollers' | 'bidirectional';
  palletSideAlongTravel: 'long' | 'short';
  /** Connected output ports (B, C1, C2) and what they lead to. */
  exits: Record<string, { target: string; gap: number }>;
}

/** Process station (S11): a short deck where one pallet at a time is processed for a time. */
export interface ProcessNode extends NodeBase {
  kind: 'process';
  length: number;
  nominalSpeed: number;
  accel: number;
  palletSideAlongTravel: 'long' | 'short';
  processingMode: 'fixed' | 'normal' | 'exponential';
  /** Fixed time, or the mean of the distribution (s). */
  processingTime: number;
  processingStdDev: number;
}

/** A port of a lift station: its id, which deck end it is at and the deck height there (m). */
export interface LiftPort {
  portId: string;
  /** At x = length (the far end); otherwise at x = 0. */
  atEnd: boolean;
  height: number;
}

/**
 * Manual lift table, scissor lift table or pallet elevator (S12–S14, ADR-0013): one deck takes a
 * pallet at the entry height, the station moves it to the exit height and gives it out.
 */
export interface LiftNode extends NodeBase {
  kind: 'lift';
  length: number;
  nominalSpeed: number;
  accel: number;
  palletSideAlongTravel: 'long' | 'short';
  entry: LiftPort;
  /** Exit ports in order of preference (the elevator prefers OUT_NEAR, S14 §1). */
  exitPorts: LiftPort[];
  upSpeed: number;
  downSpeed: number;
  verticalAccel: number;
  /** Operator side of a manual lift table (S12): checked for a source or sink there. */
  operatorPort?: string;
  /** Connected ports leading out of the station, and what they lead to. */
  exits: Record<string, { target: string; gap: number; port: string }>;
}

/** Gravity roller line (S15, ADR-0014): pallets roll down one by one to a separator and an end stop. */
export interface GravityNode extends NodeBase {
  kind: 'gravity';
  /** Plan length (m). */
  length: number;
  palletSideAlongTravel: 'long' | 'short';
  /** Fall per metre (ratio). */
  slope: number;
  /** Roller height at the entry (m), derived from the exit height and the slope. */
  entryHeight: number;
  brakeSpeed: number;
  rollingResistance: number;
  startDelay: number;
  pickupLength: number;
  separatorTime: number;
}

/** Photoelectric sensor mounted on a conveyor (S06); all positions in the host's frame (m). */
export interface SensorNode extends NodeBase {
  kind: 'sensor';
  host: string;
  sensorS: number;
  sensorLateral: number;
  reflectorS: number;
  reflectorLateral: number;
  sensorHeight: number;
  reflectorHeight: number;
  /** Height used for detection: the mean of both ends (exact for a horizontal beam). */
  beamHeight: number;
  range: number;
  responseTime: number;
  filter: number;
  activeOnDark: boolean;
  outputSignal: string;
}

/** Component whose 2D behaviour does not exist yet: it never accepts pallets. */
export interface InertNode extends NodeBase {
  kind: 'inert';
  behavior: string;
}

export type FlowNode =
  | LineNode
  | SourceNode
  | SinkNode
  | TurntableNode
  | TransferNode
  | ProcessNode
  | LiftNode
  | GravityNode
  | SensorNode
  | InertNode;

export interface FlowNetwork {
  nodes: FlowNode[];
  issues: FlowIssue[];
  /** Controller rules of the layout (A08). */
  rules: RuleData[];
}

export function buildFlowNetwork(layout: Layout, registry: DefinitionRegistry): FlowNetwork {
  const issues: FlowIssue[] = [];
  const nodes = layout.instances
    .filter((instance) => registry.has(instance.definitionId))
    // Scenery objects (S17) exist only in the 3D run.
    .filter((instance) => registry.get(instance.definitionId).behaviors.des !== NO_2D_BEHAVIOR)
    .map((instance) => {
      const definition = registry.get(instance.definitionId);
      const params = instanceParams(definition, instance.params);
      const base: NodeBase = {
        id: instance.id,
        name: instance.name,
        position: instance.position,
        yaw: instance.yaw,
        surfaceHeight: num(params, 'surfaceHeight', 0),
        downstreamGap: 0,
      };
      return createNode(definition, base, params, issues);
    });

  const byId = new Map(nodes.map((n) => [n.id, n]));
  for (const connection of layout.connections) {
    const from = byId.get(connection.from.instanceId);
    const to = byId.get(connection.to.instanceId);
    if (from === undefined || to === undefined) continue;
    if (from.kind === 'turntable') {
      const angle = STATION_PORTS[connection.from.portId];
      if (to.kind === 'turntable') {
        issues.push(
          issue(
            from,
            'turntableToTurntable',
            `pallets cannot yet pass directly to the turntable ${to.id}`,
          ),
        );
      } else if (angle !== undefined) {
        from.exits.push({
          portId: connection.from.portId,
          angle,
          target: to.id,
          gap: connection.gap,
        });
      }
      continue;
    }
    if (from.kind === 'transfer') {
      from.exits[connection.from.portId] = { target: to.id, gap: connection.gap };
      continue;
    }
    if (from.kind === 'lift') {
      from.exits[connection.from.portId] = {
        target: to.id,
        gap: connection.gap,
        port: connection.to.portId,
      };
      continue;
    }
    if (connection.from.portId !== OUTPUT_PORT[from.kind]) continue;
    from.downstream = connection.to.instanceId;
    from.downstreamPort = connection.to.portId;
    from.downstreamGap = connection.gap;
  }

  for (const node of nodes) issues.push(...connectionIssues(node, byId, nodes));
  return { nodes, issues, rules: layout.rules ?? [] };
}

const OUTPUT_PORT: Record<FlowNode['kind'], string | undefined> = {
  line: 'B',
  source: 'OUT',
  sink: undefined,
  turntable: undefined,
  transfer: undefined,
  process: 'OUT',
  lift: undefined,
  gravity: 'OUT',
  sensor: undefined,
  inert: undefined,
};

function createNode(
  definition: ComponentDefinition,
  base: NodeBase,
  params: ParamValues,
  issues: FlowIssue[],
): FlowNode {
  const behavior = definition.behaviors.des;
  if (LINE_BEHAVIORS.has(behavior)) {
    if (params['accumulationMode'] !== 'none') {
      issues.push(
        issue(
          base,
          'accumulationNotSimulated',
          `accumulation "${params['accumulationMode']}" is simulated as "none" for now`,
        ),
      );
    }
    if (params['direction'] !== undefined && params['direction'] !== 'forward') {
      issues.push(
        issue(
          base,
          'directionNotSimulated',
          `direction "${params['direction']}" is simulated as "forward" for now`,
        ),
      );
    }
    return {
      ...base,
      kind: 'line',
      length: num(params, 'length', 0),
      nominalSpeed: num(params, 'nominalSpeed', 0),
      accel: num(params, 'accel', 0),
      decel: num(params, 'decel', 0),
      minGap: num(params, 'minGap', 0),
      palletSideAlongTravel: params['palletSideAlongTravel'] === 'short' ? 'short' : 'long',
    };
  }
  if (behavior === 'des.source') {
    const palletType = PALLET_TYPES.find((p) => p.id === params['palletType']) ?? PALLET_TYPES[0];
    if (palletType === undefined) throw new Error('The library has no pallet types');
    return {
      ...base,
      kind: 'source',
      palletType,
      orientation: params['orientation'] === 'short' ? 'short' : 'long',
      loadMass: num(params, 'loadMass', 0),
      loadHeight: num(params, 'loadHeight', 0),
      generationMode: params['generationMode'] === 'exponential' ? 'exponential' : 'fixedInterval',
      interval: num(params, 'interval', 1),
      maxPallets: num(params, 'maxPallets', 0),
      blockIfOccupied: params['blockIfOccupied'] !== false,
    };
  }
  if (behavior === 'des.sink') {
    const removalTime = params['removalMode'] === 'afterTime' ? num(params, 'removalTime', 0) : 0;
    return { ...base, kind: 'sink', removalTime };
  }
  if (behavior === 'des.turntable') {
    const stations = parseStations(params['stations']);
    return {
      ...base,
      kind: 'turntable',
      platformLength: num(params, 'platformLength', 0),
      nominalSpeed: num(params, 'nominalSpeed', num(params, 'loadSpeed', 0)),
      accel: num(params, 'accel', 0),
      maxAngularSpeed: num(params, 'maxAngularSpeed', 0),
      angularAccel: num(params, 'angularAccel', 0),
      angularDecel: num(params, 'angularDecel', 0),
      settleTime: num(params, 'settleTime', 0),
      stations,
      palletSideAlongTravel: params['palletSideAlongTravel'] === 'short' ? 'short' : 'long',
      exitRule:
        params['exitRule'] === 'firstExit' || params['exitRule'] === 'random'
          ? params['exitRule']
          : 'alternate',
      exits: [],
    };
  }
  if (behavior === 'des.transfer_rc') {
    const direction = params['transferDirection'];
    const mode = params['operationMode'];
    return {
      ...base,
      kind: 'transfer',
      length: num(params, 'length', 0),
      usableWidth: num(params, 'usableWidth', 0),
      nominalSpeed: num(params, 'nominalSpeed', 0),
      accel: num(params, 'accel', 0),
      transferSpeed: num(params, 'transferSpeed', 0),
      transferAccel: num(params, 'transferAccel', 0),
      liftTime: num(params, 'liftTime', 0),
      lowerTime: num(params, 'lowerTime', 0),
      liftStroke: num(params, 'liftStroke', 0),
      transferDirection:
        direction === 'left' || direction === 'bidirectional' ? direction : 'right',
      operationMode:
        mode === 'chainsToRollers' || mode === 'bidirectional' ? mode : 'rollersToChains',
      palletSideAlongTravel: params['palletSideAlongTravel'] === 'short' ? 'short' : 'long',
      exits: {},
    };
  }
  if (behavior === 'des.process_station') {
    const mode = params['processingMode'];
    return {
      ...base,
      kind: 'process',
      length: num(params, 'length', 0),
      nominalSpeed: num(params, 'nominalSpeed', 0),
      accel: num(params, 'accel', 0),
      palletSideAlongTravel: params['palletSideAlongTravel'] === 'short' ? 'short' : 'long',
      processingMode: mode === 'normal' || mode === 'exponential' ? mode : 'fixed',
      processingTime: num(params, 'processingTime', 0),
      processingStdDev: Math.max(0, num(params, 'processingStdDev', 0)),
    };
  }
  if (behavior === 'des.lift_station') return liftNode(definition, base, params);
  if (behavior === 'des.gravity') {
    const length = num(params, 'length', 0);
    const slope = num(params, 'slope', 0);
    const exitHeight = num(params, 'exitHeight', 0);
    return {
      ...base,
      kind: 'gravity',
      length,
      palletSideAlongTravel: params['palletSideAlongTravel'] === 'short' ? 'short' : 'long',
      slope,
      entryHeight: exitHeight + slope * length,
      brakeSpeed: num(params, 'brakeSpeed', 0),
      rollingResistance: num(params, 'rollingResistance', 0),
      startDelay: Math.max(0, num(params, 'startDelay', 0)),
      pickupLength: num(params, 'pickupLength', 0),
      separatorTime: Math.max(0, num(params, 'separatorTime', 0)),
    };
  }
  if (behavior === 'des.photoeye') {
    const diffuse = params['type'] === 'diffuse';
    const sensorS = num(params, 'sensorS', 0);
    const sensorLateral = num(params, 'sensorLateral', 0);
    const sensorHeight = num(params, 'sensorHeight', 0);
    const reflectorHeight = diffuse ? sensorHeight : num(params, 'reflectorHeight', sensorHeight);
    return {
      ...base,
      kind: 'sensor',
      host: typeof params['host'] === 'string' ? params['host'] : '',
      sensorS,
      sensorLateral,
      // A diffuse sensor is treated as a beam straight across the conveyor for now.
      reflectorS: diffuse ? sensorS : num(params, 'reflectorS', sensorS),
      reflectorLateral: diffuse ? -sensorLateral : num(params, 'reflectorLateral', -sensorLateral),
      sensorHeight,
      reflectorHeight,
      beamHeight: (sensorHeight + reflectorHeight) / 2,
      range: num(params, 'range', Infinity),
      responseTime: num(params, 'responseTime', 0),
      filter: num(params, 'filter', 0),
      activeOnDark: params['polarity'] === 'activeOnDark',
      outputSignal: typeof params['outputSignal'] === 'string' ? params['outputSignal'].trim() : '',
    };
  }
  issues.push(
    issue(base, 'behaviorNotAvailable', 'has no 2D behaviour yet; pallets stop in front of it'),
  );
  return { ...base, kind: 'inert', behavior };
}

/**
 * Entry and exit ports of a lift station, with their deck heights taken from the definition's ports.
 * The manual lift table's ports work both ways: `flowDirection` makes OP (operator) or LINE the entry.
 */
function liftNode(definition: ComponentDefinition, base: NodeBase, params: ParamValues): LiftNode {
  const length = num(params, 'length', 0);
  const ports = resolvePorts(definition, params).map((port) => ({
    portId: port.id,
    atEnd: port.local.x > length / 2,
    height: port.local.y,
    direction: port.direction,
  }));
  const byId = (id: string) => ports.find((port) => port.portId === id);
  const manual = params['flowDirection'] !== undefined;
  const operator = byId('OP');
  const line = byId('LINE');
  let entry = ports.find((port) => port.direction === 'in');
  let exitPorts = ports.filter((port) => port.direction === 'out');
  if (manual && operator !== undefined && line !== undefined) {
    const out = params['flowDirection'] === 'out';
    entry = out ? line : operator;
    exitPorts = [out ? operator : line];
  }
  if (entry === undefined || exitPorts.length === 0)
    throw new Error(`${definition.id} has no entry or no exit port`);
  // A speed per direction (tables) or one speed both ways (elevator).
  const vertical = num(params, 'verticalSpeed', 0);
  return {
    ...base,
    kind: 'lift',
    length,
    nominalSpeed: num(params, 'nominalSpeed', 0),
    accel: num(params, 'accel', 0),
    palletSideAlongTravel: params['palletSideAlongTravel'] === 'short' ? 'short' : 'long',
    entry: liftPort(entry),
    exitPorts: exitPorts.map(liftPort),
    upSpeed: num(params, 'liftSpeed', vertical),
    downSpeed: num(params, 'lowerSpeed', vertical),
    verticalAccel: num(params, 'verticalAccel', 0),
    ...(manual ? { operatorPort: 'OP' } : {}),
    exits: {},
  };
}

function liftPort({ portId, atEnd, height }: LiftPort): LiftPort {
  return { portId, atEnd, height };
}

/** Station list such as "0, 90, 180, 270" (degrees) as angles in rad; invalid entries are skipped. */
function parseStations(value: unknown): number[] {
  if (typeof value !== 'string') return Object.values(STATION_PORTS);
  return value
    .split(/[,;\s]+/)
    .map(Number)
    .filter((deg) => Number.isFinite(deg))
    .map((deg) => (((deg % 360) + 360) % 360) * (Math.PI / 180));
}

function connectionIssues(
  node: FlowNode,
  byId: Map<string, FlowNode>,
  nodes: FlowNode[],
): FlowIssue[] {
  const next = node.downstream === undefined ? undefined : byId.get(node.downstream);
  if (node.kind === 'line' && next === undefined) {
    return [issue(node, 'openEnd', 'output is not connected; pallets stop at its end')];
  }
  if (node.kind === 'turntable' && node.exits.length === 0) {
    return [
      issue(
        node,
        'noExit',
        'no station is connected to the entry of another object; pallets stay on it',
      ),
    ];
  }
  if (node.kind === 'transfer') return transferIssues(node);
  if (node.kind === 'process') return processIssues(node, next);
  if (node.kind === 'lift') return liftIssues(node, byId, nodes);
  if (node.kind === 'gravity') {
    if (next === undefined)
      return [issue(node, 'noExit', 'OUT is not connected: place a sink for the operator there')];
    if (next.kind !== 'sink')
      return [
        issue(node, 'exitNotSink', 'OUT can only hand pallets to a sink (the operator) for now'),
      ];
  }
  if (node.kind === 'sensor') return sensorIssues(node, byId.get(node.host));
  if (node.kind === 'source' && next?.kind !== 'line' && next?.kind !== 'lift') {
    return [
      issue(
        node,
        'sourceNotConnected',
        'must be connected to the entry of a conveyor; it creates no pallets',
      ),
    ];
  }
  return [];
}

function processIssues(node: ProcessNode, next: FlowNode | undefined): FlowIssue[] {
  const issues: FlowIssue[] = [];
  if (next === undefined)
    issues.push(
      issue(node, 'noExit', 'OUT is not connected; the pallet stays on it after processing'),
    );
  const eur = PALLET_TYPES.find((p) => p.id === 'eur');
  const palletAlong =
    eur === undefined ? 0 : node.palletSideAlongTravel === 'long' ? eur.length : eur.width;
  if (node.length < palletAlong + MIN_DECK_MARGIN) {
    issues.push(
      issue(
        node,
        'deckTooShort',
        `the deck (${round(node.length * MM)} mm) is shorter than the pallet plus ${MIN_DECK_MARGIN * MM} mm`,
      ),
    );
  }
  return issues;
}

/** Exits of a lift station (S12–S14 §4) and the operator side of a manual table. */
function liftIssues(node: LiftNode, byId: Map<string, FlowNode>, nodes: FlowNode[]): FlowIssue[] {
  const issues: FlowIssue[] = [];
  const connectedExits = node.exitPorts.filter((port) => node.exits[port.portId] !== undefined);
  const operator = node.operatorPort;
  if (operator !== undefined) {
    const feeder = nodes.find((n) => n.downstream === node.id && n.downstreamPort === operator);
    const taker = node.exits[operator];
    if (feeder === undefined && taker === undefined) {
      issues.push(
        issue(
          node,
          'noOperator',
          `${operator} is not connected: place a source (input) or a sink (output) for the operator there`,
        ),
      );
    }
    const operatorIsEntry = node.entry.portId === operator;
    const takerKind = taker === undefined ? undefined : byId.get(taker.target)?.kind;
    if (
      (operatorIsEntry && takerKind === 'sink') ||
      (!operatorIsEntry && feeder?.kind === 'source')
    ) {
      issues.push(
        issue(
          node,
          'operatorMismatch',
          operatorIsEntry
            ? 'an input table needs a source on OP, not a sink'
            : 'an output table needs a sink on OP, not a source',
        ),
      );
    }
    if (!operatorIsEntry) return issues;
  }
  if (connectedExits.length === 0) {
    issues.push(
      issue(
        node,
        'noExit',
        `${node.exitPorts.map((p) => p.portId).join(' / ')} is not connected; the pallet stays on it`,
      ),
    );
  } else if (connectedExits.length > 1) {
    issues.push(
      issue(
        node,
        'twoExits',
        `${connectedExits.map((p) => p.portId).join(' and ')} are both connected; ${connectedExits[0]?.portId} is used`,
      ),
    );
  }
  return issues;
}

function transferIssues(node: TransferNode): FlowIssue[] {
  const issues: FlowIssue[] = [];
  const side = { right: ['C2'], left: ['C1'], bidirectional: ['C1', 'C2'] }[node.transferDirection];
  if (
    node.operationMode !== 'chainsToRollers' &&
    !side.some((port) => node.exits[port] !== undefined)
  ) {
    issues.push(
      issue(
        node,
        'noExit',
        `the chain side ${side.join('/')} is not connected; pallets from A stay on it`,
      ),
    );
  }
  if (node.operationMode !== 'rollersToChains' && node.exits['B'] === undefined) {
    issues.push(issue(node, 'noExit', 'B is not connected; pallets from the chains stay on it'));
  }
  return issues;
}

const MM = 1000;
/** Minimum clearance of a station deck beyond the pallet along travel (m). */
const MIN_DECK_MARGIN = 0.1;

function sensorIssues(node: SensorNode, host: FlowNode | undefined): FlowIssue[] {
  if (host?.kind !== 'line') {
    return [
      issue(
        node,
        'sensorWithoutHost',
        'is not mounted on a roller or chain conveyor; it detects nothing',
      ),
    ];
  }
  const issues: FlowIssue[] = [];
  if (Math.abs(node.sensorHeight - node.reflectorHeight) > 1e-9) {
    issues.push(
      issue(
        node,
        'inclinedBeam',
        `inclined beam: detection uses the mean height ${round(node.beamHeight * MM)} mm`,
      ),
    );
  }
  const length = Math.hypot(
    node.reflectorS - node.sensorS,
    node.reflectorLateral - node.sensorLateral,
    node.reflectorHeight - node.sensorHeight,
  );
  if (length > node.range) {
    issues.push(
      issue(
        node,
        'beamOutOfRange',
        `beam length ${round(length * MM)} mm exceeds the range ${round(node.range * MM)} mm`,
      ),
    );
  }
  const voidGap = largestVoid(host.palletSideAlongTravel, node.beamHeight);
  if (voidGap > 0) {
    const seconds = host.nominalSpeed > 0 ? voidGap / host.nominalSpeed : 0;
    issues.push(
      issue(
        node,
        'beamInVoidBand',
        `at ${round(node.beamHeight * MM)} mm the beam crosses pallet voids up to ${round(voidGap * MM)} mm (${seconds.toFixed(2)} s at ${host.nominalSpeed} m/s): expect several pulses per pallet; choose a continuous band or a filter`,
      ),
    );
  }
  return issues;
}

/** Longest void a horizontal beam meets in the EUR profile for the host's orientation (m). */
function largestVoid(side: 'long' | 'short', height: number): number {
  const eur = PALLET_TYPES.find((p) => p.id === 'eur');
  const band = eur?.occupancy[side === 'long' ? 'alongLength' : 'alongWidth'].find(
    (b) => height >= b.fromHeight && height < b.toHeight,
  );
  if (band === undefined) return 0;
  let largest = 0;
  for (let i = 1; i < band.solid.length; i++)
    largest = Math.max(largest, (band.solid[i]?.[0] ?? 0) - (band.solid[i - 1]?.[1] ?? 0));
  return largest;
}

function round(value: number): number {
  return Math.round(value * 10) / 10;
}

function issue(node: NodeBase, rule: string, text: string): FlowIssue {
  return { instanceId: node.id, rule, message: `${node.name || node.id}: ${text}` };
}
