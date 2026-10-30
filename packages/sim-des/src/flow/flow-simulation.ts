// 2D (discrete-event) run of a layout: sources, conveyors without accumulation, stations
// (turntables, transfer tables, process stations, lift tables and elevators), gravity lines, sinks and
// photoelectric sensors.
//
// Every event "settles" the plant at the current time:
//   1. pallets whose tail cleared a conveyor end move to the next object;
//   2. waiting sources try to place their pallet;
//   3. a front pallet that reached its conveyor's decision point asks the next object to accept it;
//      if it cannot, the conveyor holds and brakes so the pallet stops with its front at the end.
// Conveyors linked by a crossing pallet form a group that moves as one unit at the speed of the
// slowest member (T04 §9, Open Questions Q2/Q3). Then one wake-up is scheduled at the earliest
// predicted change (end of a ramp, a tail clearing an end, a front reaching a decision point).
// A station's decks are short conveyors in this scheme; its turns, lifts and lowerings are separate
// timed steps (station.ts).
import { type Rule, type SignalValues, evaluateRules } from '@intracraft/control';
import { type SimTime, US_PER_S, msToSimTime } from '@intracraft/core-model';
import { timeToReach } from '../kinematics';
import { GravityLine } from './gravity-line';
import { DesSimulator } from '../simulator';
import { EPS, LineConveyor, type LineStatus, SPEED_EPS } from './line-conveyor';
import type {
  FlowIssue,
  FlowNetwork,
  FlowNode,
  LiftNode,
  LineNode,
  ProcessNode,
  SourceNode,
  TransferNode,
  TurntableNode,
} from './network';
import { LiftStation } from './lift-station';
import type { Pallet } from './pallet';
import { Photoeye } from './photoeye';
import { ProcessStation } from './process-station';
import { PalletSink } from './sink';
import { PalletSource } from './source';
import type { DeckPath, Station } from './station';
import { Transfer } from './transfer';
import { Turntable } from './turntable';

const MAX_SETTLE_PASSES = 10_000;
/** Rule evaluations in a row at one instant before the controller is considered to oscillate. */
const MAX_CONTROL_PASSES = 100;
const SECONDS_PER_HOUR = 3600;

export interface FlowOptions {
  seed: number;
}

export interface PalletView {
  id: number;
  typeId: string;
  /** Conveyor carrying the pallet. */
  carrier: string;
  /** Front edge along the carrier's path (m from its start). */
  front: number;
  /**
   * Start and direction of the carrier's path in the carrier's own frame: zero for conveyors;
   * on a turntable the deck's entry edge and heading, which change when the platform turns.
   */
  pathX: number;
  pathY: number;
  pathZ: number;
  heading: number;
  /** Fall per metre along the path (gravity line); the pallet sinks and tilts with it. */
  slope?: number;
  speed: number;
  accel: number;
  length: number;
  width: number;
  height: number;
  loadHeight: number;
}

export type ObjectStatus =
  | LineStatus
  | 'idle'
  | 'waiting'
  | 'busy'
  | 'turning'
  | 'lifting'
  | 'lowering'
  | 'processing'
  | 'detecting'
  | 'clear';

export interface FlowSnapshot {
  /** Simulation time (µs). */
  time: SimTime;
  pallets: PalletView[];
  status: Record<string, ObjectStatus>;
  /** Controller signals (A08), e.g. `photoeye1.detected`, `roller2.run`. */
  signals: Record<string, boolean>;
  /** Deck heading of every turntable (rad, its own frame), following the platform while it turns. */
  angles: Record<string, number>;
  /** Deck height of every lift table and elevator (m above its base), following the vertical move. */
  heights: Record<string, number>;
}

/** Statistics per instance id, keyed like the definition's `statistics` list (times in s). */
export type FlowStatistics = Record<string, Record<string, number>>;

export class FlowSimulation {
  readonly issues: FlowIssue[];
  private readonly sim = new DesSimulator({ trace: false });
  private readonly nodes = new Map<string, FlowNode>();
  private readonly lines: LineConveyor[] = [];
  private readonly lineById = new Map<string, LineConveyor>();
  private readonly sources: PalletSource[] = [];
  private readonly sinks = new Map<string, PalletSink>();
  private readonly gravityLines = new Map<string, GravityLine>();
  private readonly stations = new Map<string, Station>();
  private readonly deckOwner = new Map<LineConveyor, Station>();
  private readonly sensors: Photoeye[] = [];
  private readonly rules: Rule[];
  private readonly signals = new Map<string, boolean>();
  /** Signal values at the last rule evaluation (the "previous" side of edge conditions). */
  private evaluated: SignalValues = {};
  private readonly groupOf = new Map<LineConveyor, LineConveyor[]>();
  private readonly warned = new Set<string>();
  private groups: LineConveyor[][] = [];
  private motionTicket: number | undefined;
  private nextPalletId = 1;
  private statsTime: SimTime = 0;

  constructor(network: FlowNetwork, options: FlowOptions) {
    this.issues = [...network.issues];
    this.rules = network.rules;
    for (const [index, node] of network.nodes.entries()) {
      this.nodes.set(node.id, node);
      if (node.kind === 'line') {
        const line = new LineConveyor(node);
        this.lines.push(line);
        this.lineById.set(node.id, line);
      } else if (node.kind === 'source') {
        this.sources.push(new PalletSource(node, options.seed, index + 1));
      } else if (node.kind === 'sink') {
        this.sinks.set(node.id, new PalletSink(node));
      } else if (node.kind === 'gravity') {
        this.gravityLines.set(node.id, new GravityLine(node));
      } else if (node.kind === 'sensor') {
        this.sensors.push(new Photoeye(node));
      } else if (node.kind === 'turntable' || node.kind === 'transfer' || node.kind === 'process' || node.kind === 'lift') {
        const station = createStation(node, options.seed, index + 1);
        this.stations.set(node.id, station);
        for (const deck of station.decks) {
          this.lines.push(deck);
          this.deckOwner.set(deck, station);
        }
      }
    }

    this.sim.on('source.arrival', (sim, event) => {
      const source = this.sources.find((s) => s.node.id === event.target);
      if (source !== undefined) source.dueSince = sim.now;
      this.settle();
    });
    this.sim.on('sink.free', () => this.settle());
    this.sim.on('station.done', (sim, event) => {
      // The time of the step belongs to the phase that ends now (processing, lifting…).
      this.accumulate(sim.now);
      this.stations.get(event.target)?.finishTimedStep(sim.now);
      this.settle();
    });
    this.sim.on('sensor.output', (sim, event) => {
      const sensor = this.sensors.find((s) => s.node.id === event.target);
      if (sensor === undefined) return;
      this.accumulate(sim.now);
      sensor.pendingTicket = undefined;
      sensor.commit(sensor.raw);
      this.settle();
    });
    this.sim.on('signal.write', (_sim, event) => {
      this.signals.set(event.target, event.data === true);
      this.settle();
    });
    this.sim.on('flow.motion', () => {
      this.motionTicket = undefined;
      this.settle();
    });
    this.sim.on('gravity.motion', (_sim, event) => {
      const line = this.gravityLines.get(event.target);
      if (line !== undefined) line.ticket = undefined;
      this.settle();
    });

    // Inputs start switched on; outputs start from the empty plant.
    for (const line of this.lines) if (!this.deckOwner.has(line)) this.signals.set(`${line.node.id}.run`, true);
    for (const source of this.sources) this.signals.set(`${source.node.id}.enable`, true);
    this.updateOutputs();
    this.evaluated = Object.fromEntries(this.signals);

    for (const source of this.sources) {
      if (source.active && this.lineAfter(source.node) !== undefined) {
        this.sim.schedule(0, { type: 'source.arrival', target: source.node.id });
      }
    }
  }

  get now(): SimTime {
    return this.sim.now;
  }

  runUntil(time: SimTime): void {
    this.sim.runUntil(time);
  }

  /** Time of the next event (the "step" target of the editor), or undefined when nothing will happen. */
  nextEventTime(): SimTime | undefined {
    return this.sim.nextEventTime();
  }

  snapshot(): FlowSnapshot {
    const t = this.sim.now / US_PER_S;
    const pallets: PalletView[] = this.lines.flatMap((line) => {
      const path = this.pathOf(line);
      return line.pallets.map((pallet) => ({
        id: pallet.id,
        typeId: pallet.typeId,
        carrier: line.carrier,
        front: line.frontAt(pallet, t),
        ...path,
        speed: line.speedAt(t),
        accel: line.accel,
        length: pallet.length,
        width: pallet.width,
        height: pallet.height,
        loadHeight: pallet.loadHeight,
      }));
    });
    for (const gravity of this.gravityLines.values()) {
      for (const p of gravity.pallets) {
        pallets.push({
          id: p.pallet.id,
          typeId: p.pallet.typeId,
          carrier: gravity.node.id,
          front: gravity.frontAt(p, t),
          pathX: 0,
          pathY: gravity.node.entryHeight,
          pathZ: 0,
          heading: 0,
          slope: gravity.node.slope,
          speed: gravity.speedAt(p, t),
          accel: p.stopped ? 0 : p.accel,
          length: p.pallet.length,
          width: p.pallet.width,
          height: p.pallet.height,
          loadHeight: p.pallet.loadHeight,
        });
      }
    }
    const status: Record<string, ObjectStatus> = {};
    for (const line of this.lines) if (!this.deckOwner.has(line)) status[line.node.id] = line.status;
    for (const gravity of this.gravityLines.values()) status[gravity.node.id] = gravity.status;
    for (const source of this.sources) status[source.node.id] = source.dueSince === undefined ? 'idle' : 'waiting';
    for (const sink of this.sinks.values()) status[sink.node.id] = this.sim.now < sink.busyUntil ? 'busy' : 'idle';
    const angles: Record<string, number> = {};
    const heights: Record<string, number> = {};
    for (const sensor of this.sensors) status[sensor.node.id] = sensor.detecting ? 'detecting' : 'clear';
    for (const station of this.stations.values()) {
      status[station.id] = station.busyStatus ?? deckStatus(station.decks);
      if (station instanceof Turntable) angles[station.id] = station.angleAt(this.sim.now);
      if (station instanceof LiftStation) heights[station.id] = station.heightAt(this.sim.now);
    }
    return { time: this.sim.now, pallets, status, signals: Object.fromEntries(this.signals), angles, heights };
  }

  statistics(): FlowStatistics {
    this.accumulate(this.sim.now);
    const hours = this.sim.now / US_PER_S / SECONDS_PER_HOUR;
    const perHour = (count: number) => (hours > 0 ? count / hours : 0);
    const mean = (sum: SimTime, count: number) => (count > 0 ? sum / count / US_PER_S : 0);
    const result: FlowStatistics = {};
    for (const { node, stats } of this.lines.filter((l) => !this.deckOwner.has(l))) {
      const total = stats.empty + stats.working + stats.blocked;
      result[node.id] = {
        utilization: total > 0 ? stats.working / total : 0,
        blockedTime: stats.blocked,
        emptyTime: stats.empty,
        palletsProcessed: stats.processed,
        throughputPerHour: perHour(stats.processed),
        meanTransitTime: mean(stats.transitSum, stats.processed),
        meanOccupancy: total > 0 ? stats.occupancy / total : 0,
      };
    }
    for (const { node, stats, dueSince } of this.sources) {
      const waitingNow = dueSince === undefined ? 0 : this.sim.now - dueSince;
      result[node.id] = {
        palletsCreated: stats.created,
        palletsRejected: stats.rejected,
        waitingTime: (stats.waiting + waitingNow) / US_PER_S,
      };
    }
    for (const station of this.stations.values()) result[station.id] = station.statistics();
    for (const sensor of this.sensors) result[sensor.node.id] = sensor.statistics();
    for (const gravity of this.gravityLines.values()) result[gravity.node.id] = gravity.statistics();
    for (const { node, stats } of this.sinks.values()) {
      result[node.id] = {
        palletsReceived: stats.received,
        throughputPerHour: perHour(stats.received),
        meanCycleTime: mean(stats.cycleSum, stats.received),
      };
    }
    return result;
  }

  /** Settles the plant, then lets the controller react; its immediate writes settle the plant again. */
  private settle(): void {
    for (let pass = 0; ; pass++) {
      if (pass > MAX_CONTROL_PASSES) throw new Error(`The controller rules keep changing signals at ${this.sim.now} µs`);
      this.settlePlant();
      if (!this.control()) break;
    }
  }

  private settlePlant(): void {
    const now = this.sim.now;
    this.accumulate(now);
    for (const line of this.lines) line.advanceTo(now / US_PER_S);
    for (const gravity of this.gravityLines.values()) gravity.advanceTo(now / US_PER_S);

    for (let pass = 0; ; pass++) {
      if (pass > MAX_SETTLE_PASSES) throw new Error(`Flow did not settle at ${now} µs`);
      this.regroup();
      let changed = this.completeCrossings(now);
      changed = this.feedSources(now) || changed;
      changed = this.decide(now) || changed;
      changed = this.advanceStations(now) || changed;
      changed = this.serveGravityLines(now) || changed;
      if (!changed) break;
    }

    this.updateSensors();
    for (const group of this.groups) this.planGroup(group);
    this.scheduleMotion();
    this.scheduleGravity(now);
  }

  /** The operator (sink) takes the pallet resting at a gravity line's end stop when free (S15 §2.5). */
  private serveGravityLines(now: SimTime): boolean {
    let changed = false;
    for (const gravity of this.gravityLines.values()) {
      const sink = gravity.node.downstream === undefined ? undefined : this.sinks.get(gravity.node.downstream);
      if (sink === undefined || gravity.atEndStop === undefined || !sink.accepts(now)) continue;
      sink.incoming += 1;
      sink.receive(gravity.take(now), now);
      if (sink.busyUntil > now) this.sim.schedule(sink.busyUntil - now, { type: 'sink.free', target: sink.node.id });
      changed = true;
    }
    return changed;
  }

  /** One wake-up per gravity line at its next predicted change. */
  private scheduleGravity(now: SimTime): void {
    for (const gravity of this.gravityLines.values()) {
      if (gravity.ticket !== undefined) this.sim.cancel(gravity.ticket);
      gravity.ticket = undefined;
      const next = gravity.nextChange();
      if (next === undefined) continue;
      const delay = Math.max(1, Math.round(next * US_PER_S) - now);
      gravity.ticket = this.sim.schedule(delay, { type: 'gravity.motion', target: gravity.node.id });
    }
  }

  /** Pallets whose tail cleared the conveyor end now belong to the next object. */
  private completeCrossings(now: SimTime): boolean {
    let changed = false;
    for (const line of this.lines) {
      const pallet = line.outgoing;
      if (pallet === undefined || pallet.front - pallet.length < line.node.length - EPS) continue;
      line.release(now);
      const next = this.nodeAfter(line.node);
      const gravity = next === undefined ? undefined : this.gravityLines.get(next.id);
      if (gravity !== undefined) {
        gravity.receive(pallet, line.speed, now);
      } else if (next?.kind === 'sink') {
        const sink = this.sinks.get(next.id);
        sink?.receive(pallet, now);
        if (sink !== undefined && sink.busyUntil > now) {
          this.sim.schedule(sink.busyUntil - now, { type: 'sink.free', target: sink.node.id });
        }
      } else {
        const target = this.lineAfter(line.node);
        if (target === undefined) throw new Error(`${line.node.id}: crossing pallet has no destination`);
        pallet.front -= line.node.length + line.node.downstreamGap;
        pallet.enteredAt = now;
        target.pallets.push(pallet);
        target.incoming = undefined;
      }
      changed = true;
    }
    return changed;
  }

  private feedSources(now: SimTime): boolean {
    let changed = false;
    for (const source of this.sources) {
      const line = this.lineAfter(source.node);
      if (source.dueSince === undefined || line === undefined || this.signals.get(`${source.node.id}.enable`) === false) continue;
      const { palletType, orientation } = source.node;
      const length = alongLength(source.node);
      // A source can feed a station directly (the operator at a manual lift table, S12).
      const station = this.deckOwner.get(line);
      const accepts = station === undefined || station.canReceive(source.node.downstreamPort);
      if (accepts && this.canEnter(line, length + line.node.minGap)) {
        station?.receive(source.node.downstreamPort, length, now);
        const pallet: Pallet = {
          id: this.nextPalletId++,
          typeId: palletType.id,
          along: orientation,
          length,
          width: orientation === 'long' ? palletType.width : palletType.length,
          height: palletType.height,
          loadMass: source.node.loadMass,
          loadHeight: source.node.loadHeight,
          createdAt: now,
          enteredAt: now,
          front: length,
        };
        line.pallets.push(pallet);
        this.checkOrientation(line, pallet);
        source.placed(now);
      } else if (!source.node.blockIfOccupied) {
        source.rejected();
      } else {
        continue;
      }
      if (source.active) this.sim.schedule(source.nextDelay(), { type: 'source.arrival', target: source.node.id });
      changed = true;
    }
    return changed;
  }

  /** Front pallets at their decision point either get accepted downstream or make their conveyor hold. */
  private decide(now: SimTime): boolean {
    let changed = false;
    for (const line of this.lines) {
      const pallet = line.front;
      if (pallet === undefined || line.outgoing !== undefined) continue;
      if (!line.holding && pallet.front < this.decisionPoint(line) - EPS) continue;
      if (this.reserve(line, pallet, now)) {
        line.holding = false;
        return true; // groups changed: re-evaluate everything
      }
      if (!line.holding) {
        line.holding = true;
        changed = true;
      }
    }
    return changed;
  }

  private reserve(line: LineConveyor, pallet: Pallet, now: SimTime): boolean {
    const next = this.nodeAfter(line.node);
    const gravity = next === undefined ? undefined : this.gravityLines.get(next.id);
    if (gravity !== undefined) {
      // Like a sink: the conveyor pushes the pallet in once a whole pallet fits behind the last one.
      if (!gravity.canAccept(pallet.length)) {
        gravity.waitingLength = pallet.length;
        return false;
      }
      gravity.waitingLength = undefined;
      gravity.incoming += 1;
      line.outgoing = pallet;
      this.checkOrientation(line, pallet);
      return true;
    }
    if (next?.kind === 'sink') {
      const sink = this.sinks.get(next.id);
      if (sink === undefined || !sink.accepts(now)) return false;
      sink.incoming += 1;
      line.outgoing = pallet;
      return true;
    }
    const target = this.lineAfter(line.node);
    const travel = line.node.length - (pallet.front - pallet.length) + line.node.downstreamGap;
    if (target === undefined || !this.canEnter(target, 0) || !hasRoomFor(target, travel)) return false;
    // A conveyor switched off cannot pull a pallet across.
    if (!(this.groupOf.get(target) ?? [target]).every((l) => this.runs(l))) return false;
    const station = this.deckOwner.get(target);
    if (station !== undefined) {
      if (!station.canReceive(line.node.downstreamPort)) {
        this.scheduleStep(station, station.prepareFor(line.node.downstreamPort, now), now);
        return false;
      }
      station.receive(line.node.downstreamPort, pallet.length, now);
    }
    target.incoming = pallet;
    line.outgoing = pallet;
    this.checkOrientation(target, pallet);
    return true;
  }

  /**
   * A conveyor without accumulation can take a new pallet when nothing else is entering, it is not
   * held (receiving would push its front pallet past the end), and its last pallet has moved
   * `clearance` beyond the entry.
   */
  private canEnter(line: LineConveyor, clearance: number): boolean {
    if (line.incoming !== undefined) return false;
    const rearTail = line.rearTail;
    if (rearTail === undefined) return true;
    if (this.isHeld(this.groupOf.get(line) ?? [line])) return false;
    return rearTail >= clearance - EPS;
  }

  /** Position where the front pallet must be accepted downstream, or braking starts to stop at the end. */
  private decisionPoint(line: LineConveyor): number {
    const { maxSpeed, decel } = limits(this.groupOf.get(line) ?? [line]);
    return line.end - (maxSpeed * maxSpeed) / (2 * decel);
  }

  private isHeld(group: LineConveyor[]): boolean {
    return group.some((line) => line.holding);
  }

  /** The controller has not switched the conveyor off (`<id>.run`, true unless a rule wrote false). */
  private runs(line: LineConveyor): boolean {
    return this.signals.get(`${line.node.id}.run`) !== false;
  }

  /** Groups of conveyors linked by a pallet crossing from one to the next. */
  private regroup(): void {
    const parent = this.lines.map((_, i) => i);
    const root = (i: number): number => {
      let r = i;
      while (parent[r] !== r) r = parent[r] ?? r;
      return r;
    };
    for (const [i, line] of this.lines.entries()) {
      const target = line.outgoing === undefined ? undefined : this.lineAfter(line.node);
      if (target === undefined) continue;
      const j = this.lines.indexOf(target);
      parent[root(j)] = root(i);
    }
    const byRoot = new Map<number, LineConveyor[]>();
    for (const [i, line] of this.lines.entries()) {
      const members = byRoot.get(root(i)) ?? [];
      members.push(line);
      byRoot.set(root(i), members);
    }
    this.groups = [...byRoot.values()];
    this.groupOf.clear();
    for (const group of this.groups) for (const line of group) this.groupOf.set(line, group);
  }

  /** Sets the shared speed and acceleration of a group from its pallets, holds and limits. */
  private planGroup(group: LineConveyor[]): void {
    const carriers = group.filter((line) => line.pallets.length > 0);
    const held = this.isHeld(group);
    const switchedOff = !group.every((line) => this.runs(line));
    for (const line of group) {
      line.status = line.pallets.length === 0 ? 'empty' : held ? 'blocked' : switchedOff ? 'stopped' : 'working';
    }
    if (carriers.length === 0) {
      for (const line of group) setMotion(line, 0, 0);
      return;
    }

    const { maxSpeed, accel, decel } = limits(group);
    let speed = Math.min(...carriers.map((line) => line.speed));
    let rate: number;
    if (held) {
      const room = Math.min(...group.filter((l) => l.holding).map((l) => l.end - (l.front?.front ?? 0)));
      if (speed <= SPEED_EPS || room <= EPS) {
        speed = 0;
        rate = 0;
      } else {
        rate = -Math.max(decel, (speed * speed) / (2 * room));
      }
    } else if (switchedOff) {
      if (speed <= SPEED_EPS) speed = 0;
      rate = speed === 0 ? 0 : -decel;
    } else if (speed < maxSpeed - SPEED_EPS) {
      rate = accel;
    } else if (speed > maxSpeed + SPEED_EPS) {
      rate = -decel;
    } else {
      speed = maxSpeed;
      rate = 0;
    }
    for (const line of group) setMotion(line, speed, rate);
  }

  /** One wake-up at the earliest predicted change of any group. */
  private scheduleMotion(): void {
    let earliest = Infinity;
    for (const group of this.groups) earliest = Math.min(earliest, this.nextChange(group));
    if (this.motionTicket !== undefined) this.sim.cancel(this.motionTicket);
    this.motionTicket = undefined;
    if (!Number.isFinite(earliest)) return;
    const delay = Math.max(1, Math.round(earliest * US_PER_S));
    this.motionTicket = this.sim.schedule(delay, { type: 'flow.motion', target: 'flow' });
  }

  /** Seconds until the group's next predicted change, or Infinity. */
  private nextChange(group: LineConveyor[]): number {
    const first = group[0];
    if (first === undefined || (first.speed === 0 && first.accel === 0)) return Infinity;
    const { speed, accel } = first;
    const { maxSpeed } = limits(group);
    const segment = { t0: 0, s0: 0, v0: speed, a: accel };
    const times: number[] = [];

    if (accel > 0) times.push((maxSpeed - speed) / accel);
    const toRest = this.isHeld(group) || !group.every((line) => this.runs(line));
    if (accel < 0) times.push(toRest ? speed / -accel : (speed - maxSpeed) / -accel);
    for (const line of group) {
      const pallet = line.front;
      if (pallet === undefined) continue;
      const distance =
        line.outgoing !== undefined
          ? line.node.length - (pallet.front - pallet.length)
          : line.holding
            ? undefined
            : this.decisionPoint(line) - pallet.front;
      if (distance === undefined || distance <= 0) continue;
      const time = timeToReach(segment, distance);
      if (time !== undefined) times.push(time);
    }
    for (const sensor of this.sensors) {
      const host = this.lineById.get(sensor.node.host);
      if (host === undefined || !group.includes(host)) continue;
      for (const { pallet, front } of this.palletsOver(host)) {
        const next = sensor
          .intervals(pallet)
          .flat()
          .find((boundary) => boundary > front + EPS);
        const time = next === undefined ? undefined : timeToReach(segment, next - front);
        if (time !== undefined) times.push(time);
      }
    }
    for (const line of group) {
      const needed = this.entryWaiterClearance(line);
      const rearTail = line.rearTail;
      if (needed === undefined || rearTail === undefined || rearTail >= needed - EPS) continue;
      const time = timeToReach(segment, needed - rearTail);
      if (time !== undefined) times.push(time);
    }
    return Math.min(...times.filter((t) => t >= 0));
  }

  /** Clearance behind the last pallet that a waiting feeder (source or held conveyor) needs. */
  private entryWaiterClearance(line: LineConveyor): number | undefined {
    for (const source of this.sources) {
      if (source.node.downstream !== line.node.id || source.dueSince === undefined) continue;
      return alongLength(source.node) + line.node.minGap;
    }
    const feeder = this.lines.find((l) => l.outgoing === undefined && this.lineAfter(l.node) === line);
    return feeder?.holding === true ? 0 : undefined;
  }

  /** Output signals of conveyors and sensors from the current state. */
  private updateOutputs(): void {
    for (const line of this.lines) {
      if (this.deckOwner.has(line)) continue;
      this.signals.set(`${line.node.id}.running`, line.speed > 0 || line.accel > 0);
      this.signals.set(`${line.node.id}.occupied`, line.pallets.length > 0);
    }
    for (const sensor of this.sensors) this.signals.set(sensor.signal, sensor.signalValue);
  }

  /**
   * Evaluates the rules when a signal changed since the last evaluation (A08 §2). Delayed writes
   * become events; returns true when an immediate write changed an input, so the plant must settle again.
   */
  private control(): boolean {
    if (this.rules.length === 0) return false;
    this.updateOutputs();
    const current = Object.fromEntries(this.signals);
    if (Object.keys(current).every((key) => current[key] === this.evaluated[key])) return false;
    const writes = evaluateRules(this.rules, this.evaluated, current);
    this.evaluated = current;
    let changed = false;
    for (const write of writes) {
      if (write.delayMs > 0) {
        this.sim.schedule(msToSimTime(write.delayMs), { type: 'signal.write', target: write.signal, data: write.value });
      } else if (this.signals.get(write.signal) !== write.value) {
        this.signals.set(write.signal, write.value);
        changed = true;
      }
    }
    return changed;
  }

  /** Beam changes start the sensor's response time (and filter when freed); the output follows when they elapse. */
  private updateSensors(): void {
    for (const sensor of this.sensors) {
      const host = this.lineById.get(sensor.node.host);
      const raw = host !== undefined && this.palletsOver(host).some(({ pallet, front }) => sensor.blockedBy(pallet, front));
      if (raw === sensor.raw) continue;
      sensor.raw = raw;
      if (sensor.pendingTicket !== undefined) this.sim.cancel(sensor.pendingTicket);
      sensor.pendingTicket = undefined;
      const delay = sensor.delayFor(raw);
      if (delay === 0) sensor.commit(raw);
      else sensor.pendingTicket = this.sim.schedule(delay, { type: 'sensor.output', target: sensor.node.id });
    }
  }

  /** Pallets over a conveyor with their front in its frame: its own and the one crossing onto it. */
  private palletsOver(line: LineConveyor): { pallet: Pallet; front: number }[] {
    const over = line.pallets.map((pallet) => ({ pallet, front: pallet.front }));
    const incoming = line.incoming;
    const feeder = incoming === undefined ? undefined : this.lines.find((l) => l.outgoing === incoming);
    if (incoming !== undefined && feeder !== undefined) {
      over.push({ pallet: incoming, front: incoming.front - feeder.node.length - feeder.node.downstreamGap });
    }
    return over;
  }

  /** Stations move on: a centred pallet starts a turn or a lift, an emptied deck frees the station. */
  private advanceStations(now: SimTime): boolean {
    let changed = false;
    for (const station of this.stations.values()) {
      const step = station.step(now);
      this.scheduleStep(station, step.wait, now);
      changed = step.changed || changed;
    }
    return changed;
  }

  /** Ends a station's timed step after `duration` µs, or at once when it takes no time. */
  private scheduleStep(station: Station, duration: SimTime | undefined, now: SimTime): void {
    if (duration === undefined) return;
    if (duration === 0) station.finishTimedStep(now);
    else this.sim.schedule(duration, { type: 'station.done', target: station.id });
  }

  /** Path start and heading of a carrier in its own frame (see PalletView). */
  private pathOf(line: LineConveyor): DeckPath {
    return this.deckOwner.get(line)?.pathOf(line, this.sim.now) ?? { pathX: 0, pathY: 0, pathZ: 0, heading: 0 };
  }

  private accumulate(now: SimTime): void {
    const seconds = (now - this.statsTime) / US_PER_S;
    if (seconds <= 0) return;
    for (const line of this.lines) line.accumulate(seconds);
    for (const station of this.stations.values()) station.accumulate(seconds);
    for (const sensor of this.sensors) sensor.accumulate(seconds);
    for (const gravity of this.gravityLines.values()) gravity.accumulate(seconds);
    this.statsTime = now;
  }

  private checkOrientation(line: LineConveyor, pallet: Pallet): void {
    if (pallet.along === line.node.palletSideAlongTravel || this.warned.has(line.node.id)) return;
    this.warned.add(line.node.id);
    this.issues.push({
      instanceId: line.node.id,
      rule: 'palletOrientationMismatch',
      message: `${line.node.name || line.node.id}: receives pallets with the ${pallet.along} side along travel, but is set up for the ${line.node.palletSideAlongTravel} side`,
    });
  }

  private nodeAfter(node: FlowNode): FlowNode | undefined {
    return node.downstream === undefined ? undefined : this.nodes.get(node.downstream);
  }

  /** Conveyor or station deck that receives what `node` delivers. */
  private lineAfter(node: FlowNode): LineConveyor | undefined {
    if (node.downstream === undefined) return undefined;
    return this.stations.get(node.downstream)?.entryDeck(node.downstreamPort) ?? this.lineById.get(node.downstream);
  }
}

function createStation(
  node: TurntableNode | TransferNode | ProcessNode | LiftNode,
  seed: number,
  stream: number,
): Station {
  if (node.kind === 'turntable') return new Turntable(node, seed, stream);
  if (node.kind === 'lift') return new LiftStation(node);
  return node.kind === 'transfer' ? new Transfer(node) : new ProcessStation(node, seed, stream);
}

/** Speed and ramps of a group: the most restrictive member decides. */
function limits(group: LineConveyor[]): { maxSpeed: number; accel: number; decel: number } {
  const nodes: LineNode[] = group.map((line) => line.node);
  return {
    maxSpeed: Math.min(...nodes.map((n) => n.nominalSpeed)),
    accel: Math.min(...nodes.map((n) => n.accel)),
    decel: Math.min(...nodes.map((n) => n.decel)),
  };
}

/**
 * A conveyor takes a pallet from the previous one only if the pallet can get completely aboard
 * before the conveyor's own leading pallet reaches the end: it never stops with a pallet half on.
 */
function hasRoomFor(target: LineConveyor, travel: number): boolean {
  const lead = target.pallets.find((p) => p !== target.outgoing);
  return lead === undefined || lead.front + travel <= target.node.length + EPS;
}

/** Size along travel of the pallets a source creates. */
function alongLength({ palletType, orientation }: SourceNode): number {
  return orientation === 'long' ? palletType.length : palletType.width;
}

/** Status of a station from its decks: blocked wins over working, working over empty. */
function deckStatus(decks: LineConveyor[]): LineStatus {
  if (decks.some((d) => d.status === 'blocked')) return 'blocked';
  return decks.some((d) => d.status === 'working') ? 'working' : 'empty';
}

function setMotion(line: LineConveyor, speed: number, accel: number): void {
  line.speed = speed;
  line.accel = accel;
}
