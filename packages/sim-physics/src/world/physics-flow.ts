// 3D (physics) run of a layout (A07 §3–4, ADR-0016). The layout is read exactly as in the 2D mode
// (buildFlowNetwork); every object becomes bodies in one Jolt world and the plant advances in fixed
// steps of 1/120 s:
//   1. every 10 ms the controller rules run on the current signals (A08);
//   2. the plant logic settles on the measured pallet positions: crossings complete, stations move on,
//      sources place pallets, front pallets at their decision point are accepted or hold their deck;
//   3. decks ramp their speed and set roller spin or surface velocity, platforms follow their
//      vertical profile, free rollers follow their pallet;
//   4. the engine steps; poses are read back and the photocells cast their beams.
// Arrivals, removal, sensor timing and controller rules reuse the 2D logic, so both modes differ only
// in how pallets move: here by friction, contact and impact.
import { type Rule, type SignalValues, evaluateRules } from '@intracraft/control';
import {
  type Layout,
  type SimTime,
  US_PER_S,
  type Vec3,
  createRng,
  msToSimTime,
  secondsToSimTime,
} from '@intracraft/core-model';
import { chainLayout } from '@intracraft/geometry';
import {
  type DefinitionRegistry,
  type ParamValues,
  contactPair,
  instanceParams,
  num,
} from '@intracraft/library';
import {
  type FlowIssue,
  type FlowNode,
  type FlowSnapshot,
  type FlowStatistics,
  type GravityNode,
  type LiftNode,
  type LineNode,
  type ObjectStatus,
  PalletSink,
  PalletSource,
  type PalletView,
  Photoeye,
  type ProcessNode,
  type SourceNode,
  buildFlowNetwork,
} from '@intracraft/sim-des';
import type { BodyHandle, Impact, PhysicsBackend, Pose, Shape } from '../backend';
import { DEFAULT_PHYSICS_DT } from '../fixed-step';
import { createJoltBackend } from '../jolt-backend';
import { Deck, type DeckSpec, type Exit, type SinkEnd } from './deck';
import { type DeckFrame, frameRotation, toWorld, yawQuat } from './frame';
import {
  GravityLine3D,
  type GravityRoller,
  SEPARATOR_OPEN_Y,
  SEPARATOR_THICKNESS,
} from './gravity';
import { type WorldPallet, sourceMtbh } from './pallet-body';
import { type SensorEdge, Sensor3D } from './sensor';
import { Station3D } from './station';

/** The controller runs every 10 ms (A07 §4). */
const CONTROL_PERIOD: SimTime = 10_000;
/** Position tolerance for measured positions (m). */
const POS_EPS = 5e-4;
const SPEED_EPS = 1e-6;
/** A pallet whose underside drops this far below its deck has fallen off (m). */
const FALL_DEPTH = 0.25;
const SINK_BED_LENGTH = 1.5;
const BED_THICKNESS = 0.03;
const STRAND_HEIGHT = 0.03;
const PLATE_THICKNESS = 0.02;
const SEPARATOR_HALF_HEIGHT = 0.075;
const END_STOP_TOP = 0.12;
const END_STOP_DEPTH = 0.03;
/** The pallet starts this far above the conveying surface, so it settles without overlapping (m). */
const SPAWN_LIFT = 0.002;
const FLOOR_HALF: Vec3 = { x: 250, y: 0.05, z: 250 };
const ZERO: Vec3 = { x: 0, y: 0, z: 0 };
const SECONDS_PER_HOUR = 3600;

/** Behaviours of the 3D mode that exist only in 2D for now: turntables and the transfer table. */
const NOT_IN_3D = new Set(['turntable', 'transfer', 'inert']);

export interface PhysicsFlowOptions {
  seed: number;
  /** Fixed step (s); 1/120 s by default. */
  dt?: number;
}

/** The 2D snapshot plus the real pose of every pallet (position of the underside centre, rotation). */
export interface PhysicsSnapshot extends FlowSnapshot {
  poses: Record<number, Pose>;
}

/** An impact of a pallet against something that is not conveying it (a stop, a pallet, the floor). */
export interface ImpactRecord {
  time: SimTime;
  pallet: number;
  /** What it hit: `pallet <id>`, `<instance>.endStop`, `<instance>.separator` or `floor`. */
  other: string;
  /** Approach speed (m/s) and energy ½·m·v² of the pair (J). */
  speed: number;
  energy: number;
}

/** Figures only the 3D mode measures. */
export interface PhysicsFigures {
  impacts: number;
  /** Largest impact energy of a pallet so far (J). */
  maxImpactEnergy: number;
  fallenPallets: number;
}

interface Source3D {
  logic: PalletSource;
  nextArrival: SimTime | undefined;
}

interface DelayedWrite {
  due: SimTime;
  signal: string;
  value: boolean;
}

export class PhysicsFlowSimulation {
  readonly issues: FlowIssue[];
  readonly dt: number;
  private steps = 0;
  private readonly lines: Deck[] = [];
  private readonly decks = new Map<string, Deck>();
  private readonly stations = new Map<string, Station3D>();
  private readonly stationOf = new Map<Deck, Station3D>();
  private readonly gravityLines = new Map<string, GravityLine3D>();
  private readonly sources: Source3D[] = [];
  private readonly sinks = new Map<string, SinkEnd>();
  private readonly sensors: Sensor3D[] = [];
  private readonly nodes = new Map<string, FlowNode>();
  private readonly pallets = new Map<number, WorldPallet>();
  private readonly palletBodies = new Set<BodyHandle>();
  private readonly palletOf = new Map<BodyHandle, WorldPallet>();
  /** Rollers, chain strands, platforms and sink beds: touching them is conveying, not an impact. */
  private readonly conveying = new Set<BodyHandle>();
  /** Names of the bodies a pallet may hit, for the impact log. */
  private readonly owners = new Map<BodyHandle, string>();
  private readonly impactLog: ImpactRecord[] = [];
  private readonly rules: Rule[];
  private readonly signals = new Map<string, boolean>();
  private evaluated: SignalValues = {};
  private readonly writes: DelayedWrite[] = [];
  private nextControl: SimTime = 0;
  private groups: Deck[][] = [];
  private readonly groupOf = new Map<Deck, Deck[]>();
  private readonly edges: SensorEdge[] = [];
  private readonly figures: PhysicsFigures = { impacts: 0, maxImpactEnergy: 0, fallenPallets: 0 };
  private nextPalletId = 1;

  /** Builds the world of `layout`; the engine (WASM) loads on the first call. */
  static async create(
    layout: Layout,
    registry: DefinitionRegistry,
    options: PhysicsFlowOptions,
  ): Promise<PhysicsFlowSimulation> {
    const world = await createJoltBackend({ contactPair });
    return new PhysicsFlowSimulation(layout, registry, world, options);
  }

  private constructor(
    layout: Layout,
    registry: DefinitionRegistry,
    private readonly world: PhysicsBackend,
    options: PhysicsFlowOptions,
  ) {
    this.dt = options.dt ?? DEFAULT_PHYSICS_DT;
    const network = buildFlowNetwork(layout, registry);
    this.issues = [...network.issues];
    this.rules = network.rules;
    world.addBody({
      motion: 'static',
      pose: { position: { x: 0, y: -FLOOR_HALF.y, z: 0 }, rotation: yawQuat(0) },
      shapes: [{ kind: 'box', halfExtents: FLOOR_HALF, offset: ZERO }],
      material: 'concrete',
    });
    const paramsOf = new Map(
      layout.instances
        .filter((instance) => registry.has(instance.definitionId))
        .map((instance) => {
          const definition = registry.get(instance.definitionId);
          return [
            instance.id,
            {
              params: instanceParams(definition, instance.params),
              defaults: instanceParams(definition, {}),
              behavior: definition.behaviors.des,
            },
          ];
        }),
    );
    for (const [index, node] of network.nodes.entries()) {
      this.nodes.set(node.id, node);
      const info = paramsOf.get(node.id);
      if (info === undefined) continue;
      if (node.kind === 'line')
        this.addLine(node, info.params, info.defaults, info.behavior === 'des.chain_conveyor');
      else if (node.kind === 'lift' || node.kind === 'process')
        this.addStation(node, info.params, options.seed, index + 1);
      else if (node.kind === 'gravity') this.addGravity(node, info.params);
      else if (node.kind === 'source')
        this.sources.push({
          logic: new PalletSource(node, options.seed, index + 1),
          nextArrival: undefined,
        });
      else if (node.kind === 'sink')
        this.sinks.set(node.id, { id: node.id, logic: new PalletSink(node), beds: new Map() });
      else if (NOT_IN_3D.has(node.kind)) {
        this.issues.push({
          instanceId: node.id,
          rule: 'behaviorNotAvailable3D',
          message: `${node.name || node.id}: has no 3D behaviour yet; pallets stop in front of it`,
        });
      }
    }
    for (const node of network.nodes) {
      if (node.kind === 'sensor') {
        const host = this.decks.get(node.host);
        if (host !== undefined) this.sensors.push(new Sensor3D(new Photoeye(node), host));
      }
    }
    this.connect(network.nodes);
    for (const deck of this.allDecks())
      for (const body of deck.drive.bodies) this.conveying.add(body);
    for (const gravity of this.gravityLines.values())
      for (const body of gravity.rollerBodies) this.conveying.add(body);
    for (const sink of this.sinks.values())
      for (const bed of sink.beds.values()) this.conveying.add(bed);

    for (const line of this.lines) this.signals.set(`${line.spec.id}.run`, true);
    for (const source of this.sources) this.signals.set(`${source.logic.node.id}.enable`, true);
    this.updateOutputs();
    this.evaluated = Object.fromEntries(this.signals);
    for (const source of this.sources) {
      if (source.logic.active && this.entryOf(source.logic.node) !== undefined)
        source.nextArrival = 0;
    }
  }

  get now(): SimTime {
    return Math.round(this.steps * this.dt * US_PER_S);
  }

  /** Runs whole steps until the simulated time reaches `time` (µs). */
  runUntil(time: SimTime): void {
    while (this.now < time) this.step();
  }

  /** The next instant the run can show: one step ahead. */
  nextEventTime(): SimTime {
    return Math.round((this.steps + 1) * this.dt * US_PER_S);
  }

  step(): void {
    const dt = this.dt;
    const now = this.now;
    const t = now / US_PER_S;
    this.control(now);
    this.settle(now);
    this.regroup();
    for (const group of this.groups) this.planGroup(group, dt);
    for (const deck of this.allDecks()) deck.applyDrive(this.world);
    for (const sink of this.sinks.values()) {
      for (const [deck, bed] of sink.beds) {
        const feeding = deck.exit?.kind === 'sink' && deck.exit.sink === sink;
        this.world.setSurfaceVelocity(bed, feeding ? deck.surfaceVelocity() : ZERO);
      }
    }
    for (const station of this.stations.values()) station.driveVertical(this.world, t, dt);
    for (const gravity of this.gravityLines.values()) gravity.drive(this.world, dt);

    this.world.step(dt);
    this.steps += 1;
    const after = this.now;
    for (const station of this.stations.values()) station.syncHeight(this.world, after / US_PER_S);
    this.observe();
    const stamp = now + Math.round((after - now) / 2);
    const isPallet = (body: BodyHandle) => this.palletBodies.has(body);
    for (const sensor of this.sensors) {
      sensor.sense(this.world, isPallet, stamp);
      const edge = sensor.flush(after);
      if (edge !== undefined) this.edges.push(edge);
    }
    for (const impact of this.world.takeImpacts()) this.recordImpact(impact);
    this.accumulate((after - now) / US_PER_S);
  }

  snapshot(): PhysicsSnapshot {
    const pallets: PalletView[] = [];
    const poses: Record<number, Pose> = {};
    for (const pallet of this.pallets.values()) {
      const carrier = this.decks.get(pallet.carrier) ?? this.stations.get(pallet.carrier)?.deck;
      const gravity = this.gravityLines.get(pallet.carrier);
      const e = carrier?.extent(pallet) ?? gravity?.extent(pallet);
      pallets.push({
        id: pallet.id,
        typeId: pallet.typeId,
        carrier: pallet.carrier,
        front: e?.front ?? 0,
        pathX: 0,
        pathY: 0,
        pathZ: 0,
        heading: 0,
        speed: e?.speed ?? 0,
        accel: 0,
        length: pallet.length,
        width: pallet.width,
        height: pallet.height,
        loadHeight: pallet.loadHeight,
      });
      poses[pallet.id] = pallet.pose;
    }
    const status: Record<string, ObjectStatus> = {};
    for (const line of this.lines) status[line.spec.id] = line.status;
    for (const gravity of this.gravityLines.values()) status[gravity.node.id] = gravity.status;
    for (const { logic } of this.sources)
      status[logic.node.id] = logic.dueSince === undefined ? 'idle' : 'waiting';
    for (const sink of this.sinks.values())
      status[sink.id] = this.now < sink.logic.busyUntil ? 'busy' : 'idle';
    for (const sensor of this.sensors)
      status[sensor.id] = sensor.logic.detecting ? 'detecting' : 'clear';
    const heights: Record<string, number> = {};
    for (const station of this.stations.values()) {
      status[station.id] = station.busyStatus ?? station.deck.status;
      if (station.node.kind === 'lift') heights[station.id] = station.height;
    }
    return {
      time: this.now,
      pallets,
      status,
      signals: Object.fromEntries(this.signals),
      angles: {},
      heights,
      poses,
    };
  }

  statistics(): FlowStatistics {
    const hours = this.now / US_PER_S / SECONDS_PER_HOUR;
    const perHour = (count: number) => (hours > 0 ? count / hours : 0);
    const mean = (sum: SimTime, count: number) => (count > 0 ? sum / count / US_PER_S : 0);
    const result: FlowStatistics = {};
    for (const { spec, stats } of this.lines) {
      const total = stats.empty + stats.working + stats.blocked + stats.stopped;
      result[spec.id] = {
        utilization: total > 0 ? stats.working / total : 0,
        blockedTime: stats.blocked,
        emptyTime: stats.empty,
        palletsProcessed: stats.processed,
        throughputPerHour: perHour(stats.processed),
        meanTransitTime: mean(stats.transitSum, stats.processed),
        meanOccupancy: total > 0 ? stats.occupancy / total : 0,
      };
    }
    for (const { logic } of this.sources) {
      const waitingNow = logic.dueSince === undefined ? 0 : this.now - logic.dueSince;
      result[logic.node.id] = {
        palletsCreated: logic.stats.created,
        palletsRejected: logic.stats.rejected,
        waitingTime: (logic.stats.waiting + waitingNow) / US_PER_S,
      };
    }
    for (const station of this.stations.values()) result[station.id] = station.statistics();
    for (const sensor of this.sensors) result[sensor.id] = sensor.logic.statistics();
    for (const gravity of this.gravityLines.values())
      result[gravity.node.id] = gravity.statistics();
    for (const { id, logic } of this.sinks.values()) {
      result[id] = {
        palletsReceived: logic.stats.received,
        throughputPerHour: perHour(logic.stats.received),
        meanCycleTime: mean(logic.stats.cycleSum, logic.stats.received),
      };
    }
    return result;
  }

  /** Impacts and fallen pallets so far. */
  physicsFigures(): PhysicsFigures {
    return { ...this.figures };
  }

  /** Impacts since the previous call. */
  takeImpacts(): ImpactRecord[] {
    return this.impactLog.splice(0);
  }

  /** Sensor output changes since the previous call, with their exact times. */
  takeSensorEdges(): SensorEdge[] {
    return this.edges.splice(0);
  }

  stateHash(): string {
    return this.world.stateHash();
  }

  dispose(): void {
    this.world.dispose();
  }

  // ── Building ──────────────────────────────────────────────────────────────────────────────

  private addLine(
    node: LineNode,
    params: ParamValues,
    defaults: ParamValues,
    chains: boolean,
  ): void {
    const spec = deckSpec(node, num(params, 'usableWidth', 1), node.decel, node.minGap);
    const frame: DeckFrame = {
      origin: { ...node.position, y: node.position.y + node.surfaceHeight },
      yaw: node.yaw,
      slope: 0,
    };
    const rotation = frameRotation(frame);
    const bodies: BodyHandle[] = [];
    if (chains) {
      const width = num(params, 'chainSupportWidth', 0.04);
      const friction = override(params, defaults, 'chainMu');
      for (const z of chainLayout(
        spec.usableWidth,
        Math.max(2, Math.round(num(params, 'chainCount', 3))),
      )) {
        bodies.push(
          this.world.addBody({
            motion: 'static',
            pose: {
              position: toWorld(frame, { x: spec.length / 2, y: -STRAND_HEIGHT / 2, z }),
              rotation,
            },
            shapes: [
              {
                kind: 'box',
                halfExtents: { x: spec.length / 2, y: STRAND_HEIGHT / 2, z: width / 2 },
                offset: ZERO,
              },
            ],
            material: 'steel-chain',
            ...friction,
          }),
        );
      }
      this.registerLine(new Deck(spec, frame, { kind: 'surface', bodies }));
      return;
    }
    const diameter = num(params, 'rollerDiameter', 0.089);
    const friction = override(params, defaults, 'rollerMu');
    for (const x of rollerPositions(spec.length, num(params, 'rollerPitch', 0.2063))) {
      bodies.push(
        this.world.addBody({
          motion: 'kinematic',
          pose: { position: toWorld(frame, { x, y: -diameter / 2, z: 0 }), rotation },
          shapes: [
            {
              kind: 'cylinder',
              radius: diameter / 2,
              halfLength: spec.usableWidth / 2,
              axis: 'z',
              offset: ZERO,
            },
          ],
          material: 'steel-zinc',
          ...friction,
        }),
      );
    }
    this.registerLine(new Deck(spec, frame, { kind: 'rollers', bodies, diameter }));
  }

  private registerLine(deck: Deck): void {
    this.lines.push(deck);
    this.decks.set(deck.spec.id, deck);
  }

  /** Lift tables, elevator and process station: one deck on a kinematic platform (ADR-0016). */
  private addStation(
    node: LiftNode | ProcessNode,
    params: ParamValues,
    seed: number,
    stream: number,
  ): void {
    const spec = deckSpec(node, num(params, 'usableWidth', 1), node.accel, 0);
    const startHeight = node.kind === 'lift' ? node.entry.height : node.surfaceHeight;
    const frame: DeckFrame = {
      origin: { ...node.position, y: node.position.y + startHeight },
      yaw: node.yaw,
      slope: 0,
    };
    const chains = params['deckType'] === 'chains';
    const shapes: Shape[] = [];
    if (chains) {
      const width = num(params, 'chainSupportWidth', 0.04);
      for (const z of chainLayout(
        spec.usableWidth,
        Math.max(2, Math.round(num(params, 'chainCount', 3))),
      )) {
        shapes.push({
          kind: 'box',
          halfExtents: { x: spec.length / 2, y: STRAND_HEIGHT / 2, z: width / 2 },
          offset: { x: 0, y: -STRAND_HEIGHT / 2, z },
        });
      }
    } else {
      const diameter = num(params, 'rollerDiameter', 0.089);
      for (const x of rollerPositions(spec.length, num(params, 'rollerPitch', 0.2063))) {
        shapes.push({
          kind: 'cylinder',
          radius: diameter / 2,
          halfLength: spec.usableWidth / 2,
          axis: 'z',
          offset: { x: x - spec.length / 2, y: -diameter / 2, z: 0 },
        });
      }
    }
    const platform = this.world.addBody({
      motion: 'kinematic',
      pose: {
        position: toWorld(frame, { x: spec.length / 2, y: 0, z: 0 }),
        rotation: frameRotation(frame),
      },
      shapes,
      material: chains ? 'steel-chain' : 'steel-zinc',
    });
    const deck = new Deck(spec, frame, { kind: 'surface', bodies: [platform] });
    const rng = node.kind === 'process' ? createRng(seed, stream) : undefined;
    const station = new Station3D(node, deck, platform, node.position, rng);
    deck.direction = station.entryDirection;
    this.stations.set(node.id, station);
    this.stationOf.set(deck, station);
  }

  private addGravity(node: GravityNode, params: ParamValues): void {
    const usableWidth = num(params, 'usableWidth', 1);
    const frame: DeckFrame = {
      origin: { ...node.position, y: node.position.y + node.entryHeight },
      yaw: node.yaw,
      slope: node.slope,
    };
    const rotation = frameRotation(frame);
    const diameter = num(params, 'rollerDiameter', 0.089);
    const rollers: GravityRoller[] = rollerPositions(
      node.length,
      num(params, 'rollerPitch', 0.2063),
    ).map((s) => ({
      s,
      body: this.world.addBody({
        motion: 'kinematic',
        pose: { position: toWorld(frame, { x: s, y: -diameter / 2, z: 0 }), rotation },
        shapes: [
          {
            kind: 'cylinder',
            radius: diameter / 2,
            halfLength: usableWidth / 2,
            axis: 'z',
            offset: ZERO,
          },
        ],
        material: 'steel-zinc',
      }),
    }));
    const plate = (x: number, y: number, halfY: number, halfZ: number) => ({
      pose: { position: toWorld(frame, { x, y, z: 0 }), rotation },
      shapes: [
        {
          kind: 'box' as const,
          halfExtents: { x: PLATE_THICKNESS / 2, y: halfY, z: halfZ },
          offset: ZERO,
        },
      ],
      material: 'steel-painted',
    });
    const separatorX = node.length - node.pickupLength;
    const separator = this.world.addBody({
      motion: 'kinematic',
      ...plate(
        separatorX + SEPARATOR_THICKNESS / 2,
        SEPARATOR_OPEN_Y,
        SEPARATOR_HALF_HEIGHT,
        usableWidth * 0.3,
      ),
    });
    const stopHalf = (END_STOP_TOP + END_STOP_DEPTH) / 2;
    const endStop = this.world.addBody({
      motion: 'static',
      ...plate(
        node.length + PLATE_THICKNESS / 2,
        END_STOP_TOP - stopHalf,
        stopHalf,
        usableWidth * 0.4,
      ),
    });
    this.owners.set(separator, `${node.id}.separator`);
    this.owners.set(endStop, `${node.id}.endStop`);
    this.gravityLines.set(node.id, new GravityLine3D(node, frame, rollers, diameter, separator));
  }

  /** Wires every object's output to what it feeds, and lays a receiving bed in front of each sink. */
  private connect(nodes: FlowNode[]): void {
    for (const node of nodes) {
      if (node.kind === 'line') {
        const deck = this.decks.get(node.id);
        if (deck === undefined || node.downstream === undefined) continue;
        deck.exit = this.exitTo(
          deck,
          node.downstream,
          node.downstreamPort,
          node.downstreamGap,
          true,
          node.surfaceHeight,
        );
      } else if (node.kind === 'lift') {
        const station = this.stations.get(node.id);
        if (station === undefined) continue;
        for (const port of node.exitPorts) {
          const link = node.exits[port.portId];
          if (link === undefined) continue;
          const exit = this.exitTo(
            station.deck,
            link.target,
            link.port,
            link.gap,
            port.atEnd,
            port.height,
          );
          if (exit !== undefined) station.exits.set(port.portId, exit);
        }
      } else if (node.kind === 'process') {
        const station = this.stations.get(node.id);
        if (station === undefined || node.downstream === undefined) continue;
        const exit = this.exitTo(
          station.deck,
          node.downstream,
          node.downstreamPort,
          node.downstreamGap,
          true,
          node.surfaceHeight,
        );
        if (exit !== undefined) station.exits.set('OUT', exit);
      } else if (node.kind === 'gravity') {
        const gravity = this.gravityLines.get(node.id);
        if (gravity !== undefined && node.downstream !== undefined)
          gravity.sink = this.sinks.get(node.downstream);
      }
    }
  }

  /**
   * Exit from `deck` to the object `target`. A sink gets a receiving bed beyond the deck end (at the
   * port's height, on the `atEnd` side), moving with the deck while it hands a pallet over.
   */
  private exitTo(
    deck: Deck,
    target: string,
    port: string | undefined,
    gap: number,
    atEnd: boolean,
    height: number,
  ): Exit | undefined {
    const next = this.decks.get(target) ?? this.stations.get(target)?.entryDeck(port);
    if (next !== undefined) return { kind: 'deck', deck: next, port, gap };
    const gravity = this.gravityLines.get(target);
    if (gravity !== undefined) return { kind: 'gravity', line: gravity, gap };
    const sink = this.sinks.get(target);
    if (sink === undefined) return undefined;
    const base = this.nodes.get(deck.spec.id)?.position ?? deck.frame.origin;
    const frame: DeckFrame = {
      ...deck.frame,
      origin: { ...deck.frame.origin, y: base.y + height },
    };
    const x = atEnd ? deck.spec.length + gap + SINK_BED_LENGTH / 2 : -gap - SINK_BED_LENGTH / 2;
    const bed = this.world.addBody({
      motion: 'static',
      pose: {
        position: toWorld(frame, { x, y: -BED_THICKNESS / 2, z: 0 }),
        rotation: frameRotation(frame),
      },
      shapes: [
        {
          kind: 'box',
          halfExtents: {
            x: SINK_BED_LENGTH / 2,
            y: BED_THICKNESS / 2,
            z: deck.spec.usableWidth / 2,
          },
          offset: ZERO,
        },
      ],
      material: 'steel-zinc',
    });
    sink.beds.set(deck, bed);
    return { kind: 'sink', sink, gap };
  }

  // ── Plant logic ───────────────────────────────────────────────────────────────────────────

  private settle(now: SimTime): void {
    const t = now / US_PER_S;
    this.completeCrossings(now);
    for (const station of this.stations.values()) station.step(now);
    for (const gravity of this.gravityLines.values()) {
      gravity.update(t);
      this.serveGravity(gravity, now, t);
    }
    this.feedSources(now);
    this.decide(now);
  }

  /** Pallets whose tail cleared their deck's end now belong to the next object. */
  private completeCrossings(now: SimTime): void {
    for (const deck of this.allDecks()) {
      const pallet = deck.outgoing;
      const exit = deck.exit;
      if (pallet === undefined || exit === undefined || deck.extent(pallet).rear < deck.spec.length)
        continue;
      deck.release(pallet, now);
      if (exit.kind === 'deck') {
        pallet.carrier = this.stationOf.get(exit.deck)?.id ?? exit.deck.spec.id;
        pallet.enteredAt = now;
        exit.deck.pallets.push(pallet);
        exit.deck.incoming = undefined;
      } else if (exit.kind === 'gravity') {
        exit.line.receive(pallet, now);
      } else {
        this.removePallet(pallet);
        exit.sink.logic.receive(pallet, now);
      }
    }
  }

  /** The operator takes the pallet resting at the end stop when free (S15 §2.5). */
  private serveGravity(gravity: GravityLine3D, now: SimTime, t: number): void {
    const sink = gravity.sink;
    if (sink === undefined || gravity.atEndStop === undefined || !sink.logic.accepts(now)) return;
    sink.logic.incoming += 1;
    const pallet = gravity.take(now, t);
    this.removePallet(pallet);
    sink.logic.receive(pallet, now);
  }

  private feedSources(now: SimTime): void {
    for (const source of this.sources) {
      const { logic } = source;
      if (
        logic.dueSince === undefined &&
        source.nextArrival !== undefined &&
        source.nextArrival <= now
      ) {
        logic.dueSince = source.nextArrival;
        source.nextArrival = undefined;
      }
      const entry = this.entryOf(logic.node);
      if (
        logic.dueSince === undefined ||
        entry === undefined ||
        this.signals.get(`${logic.node.id}.enable`) === false
      )
        continue;
      const { palletType, orientation } = logic.node;
      const length = orientation === 'long' ? palletType.length : palletType.width;
      const accepts =
        entry.station === undefined || entry.station.canReceive(logic.node.downstreamPort);
      if (accepts && this.canEnter(entry.deck, length + entry.deck.spec.minGap)) {
        entry.station?.receive(length, now);
        this.spawn(logic.node, entry.deck, length, now);
        logic.placed(now);
      } else if (!logic.node.blockIfOccupied) {
        logic.rejected();
      } else {
        continue;
      }
      if (logic.active) source.nextArrival = now + logic.nextDelay();
    }
  }

  /** A new pallet at rest on the entry deck, its rear at the entry end, aligned with travel (S09 §3D). */
  private spawn(node: SourceNode, deck: Deck, length: number, now: SimTime): void {
    const mtbh = sourceMtbh(node);
    const { palletType, orientation } = node;
    const travel = length / 2;
    const x = deck.direction > 0 ? travel : deck.spec.length - travel;
    const pose: Pose = {
      position: toWorld(deck.frame, { x, y: SPAWN_LIFT, z: 0 }),
      rotation: yawQuat(deck.frame.yaw + (orientation === 'long' ? 0 : Math.PI / 2)),
    };
    const body = this.world.addBody({
      motion: 'dynamic',
      pose,
      shapes: mtbh.shapes,
      material: mtbh.material,
      massProperties: mtbh.massProperties,
      ccd: true,
    });
    // At rest on the deck: it moves with the conveying surface from the start, as in 2D.
    const velocity = deck.surfaceVelocity();
    this.world.setVelocity(body, velocity, ZERO);
    const pallet: WorldPallet = {
      id: this.nextPalletId++,
      typeId: palletType.id,
      along: orientation,
      length,
      width: orientation === 'long' ? palletType.width : palletType.length,
      height: palletType.height,
      loadMass: node.loadMass,
      loadHeight: node.loadHeight,
      createdAt: now,
      enteredAt: now,
      front: length,
      body,
      mtbh,
      carrier: this.stationOf.get(deck)?.id ?? deck.spec.id,
      pose,
      velocity: { linear: velocity, angular: ZERO },
    };
    this.pallets.set(pallet.id, pallet);
    this.palletBodies.add(body);
    this.palletOf.set(body, pallet);
    deck.pallets.push(pallet);
  }

  /** Front pallets at their decision point either get accepted downstream or make their deck hold. */
  private decide(now: SimTime): void {
    for (const deck of this.allDecks()) {
      if (deck.outgoing !== undefined) continue;
      const lead = deck.lead;
      if (lead === undefined) {
        deck.holding = false;
        continue;
      }
      if (!deck.holding && deck.extent(lead).front < this.decisionPoint(deck) - POS_EPS) continue;
      const accepted = this.reserve(deck, lead, now);
      deck.holding = !accepted;
      if (accepted) this.regroup();
    }
  }

  private reserve(deck: Deck, pallet: WorldPallet, now: SimTime): boolean {
    const exit = deck.exit;
    if (exit === undefined) return false;
    if (exit.kind === 'gravity') {
      if (!exit.line.canAccept(pallet.length + exit.gap)) {
        exit.line.waitingLength = pallet.length;
        return false;
      }
      exit.line.waitingLength = undefined;
      exit.line.entering.push(pallet);
      deck.outgoing = pallet;
      return true;
    }
    if (exit.kind === 'sink') {
      if (!exit.sink.logic.accepts(now)) return false;
      exit.sink.logic.incoming += 1;
      deck.outgoing = pallet;
      return true;
    }
    const target = exit.deck;
    const travel = deck.spec.length - deck.extent(pallet).rear + exit.gap;
    if (!this.canEnter(target, 0) || !hasRoomFor(target, travel)) return false;
    // A conveyor switched off cannot pull a pallet across.
    if (!(this.groupOf.get(target) ?? [target]).every((d) => this.runs(d))) return false;
    const station = this.stationOf.get(target);
    if (station !== undefined) {
      if (!station.canReceive(exit.port)) return false;
      station.receive(pallet.length, now);
    }
    target.incoming = pallet;
    deck.outgoing = pallet;
    return true;
  }

  /**
   * A deck can take a new pallet when nothing else is entering, it is not held and its last pallet
   * has moved `clearance` beyond the entry (ADR-0007 §5).
   */
  private canEnter(deck: Deck, clearance: number): boolean {
    if (deck.incoming !== undefined) return false;
    const rear = deck.rearTail;
    if (rear === undefined) return true;
    if ((this.groupOf.get(deck) ?? [deck]).some((d) => d.holding)) return false;
    return rear >= clearance - POS_EPS;
  }

  private decisionPoint(deck: Deck): number {
    const { maxSpeed, decel } = limits(this.groupOf.get(deck) ?? [deck]);
    return deck.end - (maxSpeed * maxSpeed) / (2 * decel);
  }

  /** The controller has not switched the deck off (`<id>.run`; station decks always run). */
  private runs(deck: Deck): boolean {
    return this.stationOf.has(deck) || this.signals.get(`${deck.spec.id}.run`) !== false;
  }

  /** Decks linked by a pallet crossing from one to the next move as one unit (ADR-0007 §3). */
  private regroup(): void {
    const decks = this.allDecks();
    const parent = decks.map((_, i) => i);
    const root = (i: number): number => {
      let r = i;
      while (parent[r] !== r) r = parent[r] ?? r;
      return r;
    };
    for (const [i, deck] of decks.entries()) {
      const exit = deck.exit;
      if (deck.outgoing === undefined || exit?.kind !== 'deck') continue;
      parent[root(decks.indexOf(exit.deck))] = root(i);
    }
    const byRoot = new Map<number, Deck[]>();
    for (const [i, deck] of decks.entries()) {
      const members = byRoot.get(root(i)) ?? [];
      members.push(deck);
      byRoot.set(root(i), members);
    }
    this.groups = [...byRoot.values()];
    this.groupOf.clear();
    for (const group of this.groups) for (const deck of group) this.groupOf.set(deck, group);
  }

  /** Shared speed of a group for the next step: ramp up, brake to stop at the end, or stop (ADR-0007 §4). */
  private planGroup(group: Deck[], dt: number): void {
    const carriers = group.filter((d) => d.pallets.length > 0);
    const held = group.some((d) => d.holding);
    const switchedOff = !group.every((d) => this.runs(d));
    for (const deck of group) {
      deck.status =
        deck.pallets.length === 0
          ? 'empty'
          : held
            ? 'blocked'
            : switchedOff
              ? 'stopped'
              : 'working';
    }
    const active =
      carriers.length > 0 ||
      group.some((d) => d.incoming !== undefined || d.outgoing !== undefined);
    if (!active) {
      for (const deck of group) [deck.speed, deck.rate] = [0, 0];
      return;
    }
    const { maxSpeed, accel, decel } = limits(group);
    const moving = carriers.length > 0 ? carriers : group;
    let speed = Math.min(...moving.map((d) => d.speed));
    let rate: number;
    if (held) {
      const room = Math.min(
        ...group
          .filter((d) => d.holding)
          .map((d) => {
            const lead = d.lead;
            return lead === undefined ? Infinity : d.end - d.extent(lead).front;
          }),
      );
      rate =
        speed <= SPEED_EPS || room <= POS_EPS ? 0 : -Math.max(decel, (speed * speed) / (2 * room));
      if (rate === 0) speed = 0;
    } else if (switchedOff) {
      rate = speed <= SPEED_EPS ? 0 : -decel;
    } else if (speed < maxSpeed - SPEED_EPS) {
      rate = accel;
    } else if (speed > maxSpeed + SPEED_EPS) {
      rate = -decel;
    } else {
      [speed, rate] = [maxSpeed, 0];
    }
    let next = speed + rate * dt;
    if (rate > 0) next = Math.min(next, maxSpeed);
    else if (rate < 0) next = Math.max(next, held || switchedOff ? 0 : maxSpeed);
    for (const deck of group) [deck.speed, deck.rate] = [next, rate];
  }

  // ── Controller (A08) ──────────────────────────────────────────────────────────────────────

  /** Applies due delayed writes and, on each 10 ms boundary, evaluates the rules on changed signals. */
  private control(now: SimTime): void {
    for (let i = this.writes.length - 1; i >= 0; i--) {
      const write = this.writes[i];
      if (write === undefined || write.due > now) continue;
      this.signals.set(write.signal, write.value);
      this.writes.splice(i, 1);
    }
    if (now < this.nextControl) return;
    while (this.nextControl <= now) this.nextControl += CONTROL_PERIOD;
    if (this.rules.length === 0) return;
    this.updateOutputs();
    const current = Object.fromEntries(this.signals);
    if (Object.keys(current).every((key) => current[key] === this.evaluated[key])) return;
    const writes = evaluateRules(this.rules, this.evaluated, current);
    this.evaluated = current;
    for (const write of writes) {
      if (write.delayMs > 0)
        this.writes.push({
          due: now + msToSimTime(write.delayMs),
          signal: write.signal,
          value: write.value,
        });
      else this.signals.set(write.signal, write.value);
    }
  }

  private updateOutputs(): void {
    for (const line of this.lines) {
      this.signals.set(`${line.spec.id}.running`, line.speed > 0 || line.rate > 0);
      this.signals.set(`${line.spec.id}.occupied`, line.pallets.length > 0);
    }
    for (const sensor of this.sensors)
      this.signals.set(sensor.logic.signal, sensor.logic.signalValue);
  }

  // ── Observation and bookkeeping ───────────────────────────────────────────────────────────

  /** Reads every pallet's pose and velocity; pallets that dropped off their deck are taken out. */
  private observe(): void {
    for (const pallet of this.pallets.values()) {
      pallet.pose = this.world.getPose(pallet.body);
      pallet.velocity = this.world.getVelocity(pallet.body);
    }
    for (const pallet of [...this.pallets.values()]) {
      const deck = this.decks.get(pallet.carrier) ?? this.stations.get(pallet.carrier)?.deck;
      const gravity = this.gravityLines.get(pallet.carrier);
      const lift = (deck?.extent(pallet) ?? gravity?.extent(pallet))?.lift ?? 0;
      if (lift > -FALL_DEPTH) continue;
      this.figures.fallenPallets += 1;
      this.issues.push({
        instanceId: pallet.carrier,
        rule: 'palletFell',
        message: `pallet ${pallet.id} fell off ${pallet.carrier} at ${(this.now / US_PER_S).toFixed(2)} s`,
      });
      for (const d of this.allDecks()) {
        const index = d.pallets.indexOf(pallet);
        if (index >= 0) d.pallets.splice(index, 1);
        if (d.incoming === pallet) d.incoming = undefined;
        if (d.outgoing === pallet) d.outgoing = undefined;
      }
      this.removePallet(pallet);
    }
  }

  private recordImpact({ time, bodies, speed, energy }: Impact): void {
    const [a, b] = bodies;
    if (this.conveying.has(a) || this.conveying.has(b)) return;
    const [palletBody, otherBody] = this.palletBodies.has(a) ? [a, b] : [b, a];
    const pallet = this.palletOf.get(palletBody);
    if (pallet === undefined) return;
    this.figures.impacts += 1;
    this.figures.maxImpactEnergy = Math.max(this.figures.maxImpactEnergy, energy);
    const other = this.palletOf.get(otherBody);
    this.impactLog.push({
      time: secondsToSimTime(time),
      pallet: pallet.id,
      other: other === undefined ? (this.owners.get(otherBody) ?? 'floor') : `pallet ${other.id}`,
      speed,
      energy,
    });
  }

  private removePallet(pallet: WorldPallet): void {
    this.world.removeBody(pallet.body);
    this.pallets.delete(pallet.id);
    this.palletBodies.delete(pallet.body);
    this.palletOf.delete(pallet.body);
  }

  private accumulate(seconds: number): void {
    for (const line of this.lines) line.accumulate(seconds);
    for (const station of this.stations.values()) station.accumulate(seconds);
    for (const sensor of this.sensors) sensor.logic.accumulate(seconds);
    for (const gravity of this.gravityLines.values()) gravity.accumulate(seconds);
  }

  /** The deck (and its station) that receives what a source creates. */
  private entryOf(node: SourceNode): { deck: Deck; station: Station3D | undefined } | undefined {
    if (node.downstream === undefined) return undefined;
    const line = this.decks.get(node.downstream);
    if (line !== undefined) return { deck: line, station: undefined };
    const station = this.stations.get(node.downstream);
    const deck = station?.entryDeck(node.downstreamPort);
    return station === undefined || deck === undefined ? undefined : { deck, station };
  }

  private allDecks(): Deck[] {
    return [...this.lines, ...[...this.stations.values()].map((s) => s.deck)];
  }
}

function deckSpec(
  node: LineNode | LiftNode | ProcessNode,
  usableWidth: number,
  decel: number,
  minGap: number,
): DeckSpec {
  return {
    id: node.id,
    name: node.name,
    length: node.length,
    usableWidth,
    nominalSpeed: node.nominalSpeed,
    accel: node.accel,
    decel,
    minGap,
    palletSideAlongTravel: node.palletSideAlongTravel,
  };
}

/** Roller axes at the pitch, the first half a pitch from the entry (as the bench and the 3D model). */
function rollerPositions(length: number, pitch: number): number[] {
  const count = Math.floor(length / pitch + 1e-9);
  return Array.from({ length: count }, (_, i) => pitch / 2 + i * pitch);
}

/** A friction changed from the definition default replaces the pair table for this conveyor (S16). */
function override(params: ParamValues, defaults: ParamValues, key: string): { friction?: number } {
  const value = params[key];
  return typeof value === 'number' && value !== defaults[key] ? { friction: value } : {};
}

/** Speed and ramps of a group: the most restrictive member decides. */
function limits(group: Deck[]): { maxSpeed: number; accel: number; decel: number } {
  return {
    maxSpeed: Math.min(...group.map((d) => d.spec.nominalSpeed)),
    accel: Math.min(...group.map((d) => d.spec.accel)),
    decel: Math.min(...group.map((d) => d.spec.decel)),
  };
}

/** A deck takes a pallet only if it gets completely aboard before the deck's own lead pallet reaches the end. */
function hasRoomFor(target: Deck, travel: number): boolean {
  const lead = target.lead;
  return lead === undefined || target.extent(lead).front + travel <= target.spec.length + POS_EPS;
}
