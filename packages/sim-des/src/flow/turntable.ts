// 2D behaviour of the chain and roller turntables (S03, S04, T13). The platform's chains or rollers
// are a short conveyor (the "deck") whose direction changes with the platform. Cycle:
//   idle → [positioning: turn to face the entry] → receiving (the pallet stops centred)
//        → turning to the exit station + settle → discharging → idle.
// The deck conveys in both directions, so a station pair (k, k + 180°) needs no turn between them.
import { type Rng, type SimTime, US_PER_S, createRng, secondsToSimTime } from '@intracraft/core-model';
import { profilePositionAt, profileTime } from '../kinematics';
import { EPS, type LineConveyor } from './line-conveyor';
import type { TurntableExit, TurntableNode } from './network';
import { type DeckPath, type Station, type StationStep, StationCounters, createDeck } from './station';

const FULL_TURN = 2 * Math.PI;
const ANGLE_EPS = 1e-6;

export type TurntablePhase = 'idle' | 'positioning' | 'receiving' | 'turning' | 'discharging';

interface Turn {
  start: SimTime;
  /** Platform angle at the start and signed rotation (rad). */
  from: number;
  rotation: number;
  /** Heading of the deck once the turn is over. */
  heading: number;
}

export class Turntable implements Station {
  readonly id: string;
  readonly deck: LineConveyor;
  readonly decks: LineConveyor[];
  phase: TurntablePhase = 'idle';
  /** Direction the deck conveys towards (rad, turntable frame). */
  heading = 0;
  /** Exit chosen for the pallet on the deck. */
  exit: TurntableExit | undefined;
  /** Station port the current pallet came in through. */
  entryPort: string | undefined;
  turns = 0;
  private turn: Turn | undefined;
  private alternateIndex = 0;
  private readonly rng: Rng;
  private readonly counters = new StationCounters();

  constructor(
    readonly node: TurntableNode,
    seed: number,
    stream: number,
  ) {
    this.id = node.id;
    this.rng = createRng(seed, stream);
    this.deck = createDeck(node, node.id, {
      length: node.platformLength,
      speed: node.nominalSpeed,
      accel: node.accel,
      side: node.palletSideAlongTravel,
    });
    this.decks = [this.deck];
  }

  entryDeck(portId: string | undefined): LineConveyor | undefined {
    return this.stationAngle(portId) === undefined ? undefined : this.deck;
  }

  get busyStatus(): 'turning' | undefined {
    return this.phase === 'positioning' || this.phase === 'turning' ? 'turning' : undefined;
  }

  /** Station angle of a port, if the port is a station the platform may stop at. */
  stationAngle(portId: string | undefined): number | undefined {
    const angle = { P0: 0, P90: Math.PI / 2, P180: Math.PI, P270: (3 * Math.PI) / 2 }[portId ?? ''];
    if (angle === undefined || !this.node.stations.some((s) => sameAngle(s, angle))) return undefined;
    return angle;
  }

  /** Free, empty and lined up with the entry: a pallet coming through `portId` can enter now. */
  canReceive(portId: string | undefined): boolean {
    const angle = this.stationAngle(portId);
    if (angle === undefined || this.phase !== 'idle' || this.deck.pallets.length > 0) return false;
    return sameLine(this.heading, angle);
  }

  /** A pallet waits at `portId`: if idle, empty and not lined up, turn to it. Returns the turn time (µs). */
  prepareFor(portId: string | undefined, now: SimTime): SimTime | undefined {
    const angle = this.stationAngle(portId);
    if (angle === undefined || this.phase !== 'idle' || this.deck.pallets.length > 0) return undefined;
    if (sameLine(this.heading, angle)) return undefined;
    this.phase = 'positioning';
    return this.startTurn(normalize(angle + Math.PI), now);
  }

  /** The pallet was accepted through `portId`: it will stop with its centre on the platform centre. */
  receive(portId: string | undefined, palletLength: number, now: SimTime): void {
    this.counters.startCycle(now);
    const angle = this.stationAngle(portId);
    if (angle !== undefined) this.heading = normalize(angle + Math.PI);
    this.phase = 'receiving';
    this.entryPort = portId;
    this.deck.end = (this.node.platformLength + palletLength) / 2;
    delete this.deck.node.downstream;
  }

  /** Centred pallet: choose the exit and turn to it. Emptied deck: free again. */
  step(now: SimTime): StationStep {
    const pallet = this.deck.front;
    const centred = pallet !== undefined && this.deck.incoming === undefined && this.deck.holding && pallet.front >= this.deck.end - EPS;
    if (this.phase === 'receiving' && centred) {
      const exit = this.chooseExit();
      if (exit === undefined) return { changed: false };
      this.exit = exit;
      this.phase = 'turning';
      return { changed: true, wait: this.startTurn(exit.angle, now) };
    }
    if (this.phase === 'discharging' && this.deck.pallets.length === 0) {
      this.phase = 'idle';
      this.exit = undefined;
      this.entryPort = undefined;
      this.counters.endCycle(now);
      return { changed: true };
    }
    return { changed: false };
  }

  /** The platform reached its station (and settled). */
  finishTimedStep(): void {
    if (this.turn !== undefined) this.heading = this.turn.heading;
    this.turn = undefined;
    if (this.phase === 'positioning') this.phase = 'idle';
    else if (this.phase === 'turning') this.startDischarge();
  }

  /** Heading of the deck at time `t` (µs), following the angular profile during a turn. */
  angleAt(t: SimTime): number {
    const turn = this.turn;
    if (turn === undefined) return this.heading;
    const elapsed = (t - turn.start) / US_PER_S;
    const covered = profilePositionAt(this.turnProfile(Math.abs(turn.rotation)), elapsed);
    return turn.from + Math.sign(turn.rotation) * covered;
  }

  pathOf(_deck: LineConveyor, now: SimTime): DeckPath {
    const heading = this.angleAt(now);
    const half = this.node.platformLength / 2;
    // Entry edge = centre − half · direction(heading), direction(h) = (cos h, −sin h) (right-hand about +Y).
    return { pathX: -half * Math.cos(heading), pathY: 0, pathZ: half * Math.sin(heading), heading };
  }

  accumulate(seconds: number): void {
    this.counters.accumulate(seconds, this.phase !== 'idle');
  }

  statistics(): Record<string, number> {
    return {
      utilization: this.counters.utilization,
      turns: this.turns,
      meanCycleTime: this.counters.meanCycle,
      palletsProcessed: this.counters.cycles,
    };
  }

  private startDischarge(): void {
    const exit = this.exit;
    const pallet = this.deck.front;
    if (exit === undefined || pallet === undefined) return;
    this.phase = 'discharging';
    // Seen from the new heading the centred pallet is again centred on the deck.
    pallet.front = (this.node.platformLength + pallet.length) / 2;
    this.deck.end = this.node.platformLength;
    this.deck.holding = false;
    this.deck.node.downstream = exit.target;
    this.deck.node.downstreamGap = exit.gap;
  }

  /** Turns the shortest way to the line of `target` (the deck runs both ways); returns the time incl. settling. */
  private startTurn(target: number, now: SimTime): SimTime {
    let rotation = normalize(target - this.heading);
    if (rotation > Math.PI) rotation -= FULL_TURN; // (−π, π]
    if (Math.abs(rotation) > Math.PI / 2 + ANGLE_EPS) rotation -= Math.sign(rotation) * Math.PI; // [−π/2, π/2]
    if (Math.abs(rotation) < ANGLE_EPS) {
      this.heading = target;
      return 0;
    }
    this.turns += 1;
    this.turn = { start: now, from: this.heading, rotation, heading: target };
    return secondsToSimTime(profileTime(this.turnProfile(Math.abs(rotation))) + this.node.settleTime);
  }

  private turnProfile(angle: number) {
    return {
      distance: angle,
      maxSpeed: this.node.maxAngularSpeed,
      accel: this.node.angularAccel,
      decel: this.node.angularDecel,
    };
  }

  /** Exits other than the entry, ordered clockwise (seen from above) starting after the entry. */
  private chooseExit(): TurntableExit | undefined {
    const entry = this.stationAngle(this.entryPort) ?? 0;
    const candidates = this.node.exits
      .filter((e) => e.portId !== this.entryPort && this.stationAngle(e.portId) !== undefined)
      .sort((a, b) => clockwiseFrom(entry, a.angle) - clockwiseFrom(entry, b.angle));
    if (candidates.length === 0) return undefined;
    switch (this.node.exitRule) {
      case 'firstExit':
        return candidates[0];
      case 'random':
        return candidates[Math.floor(this.rng.next() * candidates.length)];
      case 'alternate':
        return candidates[this.alternateIndex++ % candidates.length];
    }
  }
}

function normalize(angle: number): number {
  return ((angle % FULL_TURN) + FULL_TURN) % FULL_TURN;
}

/** Both angles lie on the same line through the platform centre (equal or opposite). */
function sameLine(a: number, b: number): boolean {
  return sameAngle(a, b) || sameAngle(a, b + Math.PI);
}

function sameAngle(a: number, b: number): boolean {
  const d = normalize(a - b);
  return d < ANGLE_EPS || FULL_TURN - d < ANGLE_EPS;
}

/** Clockwise angle from `from` to `to` seen from above (positive angles turn counter-clockwise). */
function clockwiseFrom(from: number, to: number): number {
  return normalize(from - to);
}
