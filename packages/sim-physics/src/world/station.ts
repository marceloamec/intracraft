// 3D behaviour of the stations with one deck on a kinematic platform: the manual lift table, the
// scissor lift table and the pallet elevator (S12–S14) and the process station (S11). Same cycle as
// in 2D (ADR-0013): receive and centre the pallet, lift or process it, discharge it through the exit
// port, return empty. The platform carries the pallet up and down by contact.
import {
  type Rng,
  type SimTime,
  type Vec3,
  US_PER_S,
  secondsToSimTime,
} from '@intracraft/core-model';
import type { LiftNode, LiftPort, ProcessNode } from '@intracraft/sim-des';
import type { BodyHandle, PhysicsBackend } from '../backend';
import type { Deck, Exit } from './deck';

/** A centred pallet slower than this has come to rest on the deck (m/s). */
const REST_SPEED = 0.005;
/** The pallet's rear may still hang this far over the entry end when it is centred (m). */
const ENTRY_TOLERANCE = 0.01;
const ZERO: Vec3 = { x: 0, y: 0, z: 0 };

export type StationPhase =
  'idle' | 'receiving' | 'moving' | 'processing' | 'discharging' | 'returning';

/** A vertical move with a trapezoidal profile that starts and ends at rest (ADR-0013 §3). */
export interface VerticalMove {
  from: number;
  to: number;
  start: number;
  duration: number;
  speed: number;
  accel: number;
}

export class Station3D {
  phase: StationPhase = 'idle';
  /** Deck surface height above the instance base (m). */
  height: number;
  /** Connected output ports and where they lead. */
  readonly exits = new Map<string, Exit>();
  /** Seconds working (lifting, processing…), blocked while discharging and in total; finished cycles and moves. */
  readonly stats = {
    working: 0,
    blocked: 0,
    total: 0,
    cycles: 0,
    cycleSum: 0,
    done: 0,
    processingSum: 0,
  };
  private readonly entry: LiftPort;
  private readonly exitPorts: LiftPort[];
  private move: VerticalMove | undefined;
  private waitUntil: SimTime | undefined;
  private cycleStart: SimTime = 0;

  constructor(
    readonly node: LiftNode | ProcessNode,
    readonly deck: Deck,
    private readonly platform: BodyHandle,
    /** World position of the instance (its base). */
    private readonly base: Vec3,
    private readonly rng: Rng | undefined,
  ) {
    if (node.kind === 'lift') {
      this.entry = node.entry;
      this.exitPorts = node.exitPorts;
    } else {
      this.entry = { portId: 'IN', atEnd: false, height: node.surfaceHeight };
      this.exitPorts = [{ portId: 'OUT', atEnd: true, height: node.surfaceHeight }];
    }
    this.height = this.entry.height;
  }

  get id(): string {
    return this.node.id;
  }

  /** The deck that receives pallets arriving at `portId`, if that port is the entry. */
  entryDeck(portId: string | undefined): Deck | undefined {
    return portId === this.entry.portId ? this.deck : undefined;
  }

  /** The entry deck whose travel starts at the entry port. */
  get entryDirection(): 1 | -1 {
    return this.entry.atEnd ? -1 : 1;
  }

  canReceive(portId: string | undefined): boolean {
    return (
      this.entryDeck(portId) !== undefined &&
      this.phase === 'idle' &&
      this.move === undefined &&
      this.deck.pallets.length === 0 &&
      this.deck.incoming === undefined
    );
  }

  /** A pallet of `length` is coming in: the deck runs it in and stops it centred. */
  receive(length: number, now: SimTime): void {
    this.cycleStart = now;
    this.phase = 'receiving';
    this.deck.direction = this.entryDirection;
    this.deck.end = (this.deck.spec.length + length) / 2;
    this.deck.exit = undefined;
  }

  /** Moves the cycle on at `now`; returns true when the phase changed. */
  step(now: SimTime): boolean {
    const t = now / US_PER_S;
    switch (this.phase) {
      case 'receiving': {
        if (!this.centred()) return false;
        if (this.node.kind === 'process') {
          this.phase = 'processing';
          const time = this.drawProcessingTime();
          this.stats.processingSum += time;
          this.waitUntil = now + time;
        } else {
          this.phase = 'moving';
          this.startMove(this.exitPort().height, t);
        }
        return true;
      }
      case 'processing':
        if (this.waitUntil === undefined || now < this.waitUntil) return false;
        this.waitUntil = undefined;
        this.stats.done += 1;
        this.discharge();
        return true;
      case 'moving':
        if (this.move !== undefined) return false;
        this.discharge();
        return true;
      case 'discharging':
        if (this.deck.pallets.length > 0 || this.deck.outgoing !== undefined) return false;
        this.deck.end = this.deck.spec.length;
        if (this.node.kind === 'process') {
          this.endCycle(now);
        } else {
          this.stats.done += 1;
          this.phase = 'returning';
          this.startMove(this.entry.height, t);
        }
        return true;
      case 'returning':
        if (this.move !== undefined) return false;
        this.endCycle(now);
        this.deck.direction = this.entryDirection;
        return true;
      case 'idle':
        return false;
    }
  }

  /** Sets the platform's vertical speed so that it follows the profile over the next step. */
  driveVertical(world: PhysicsBackend, t: number, dt: number): void {
    const move = this.move;
    if (move === undefined) {
      world.setVelocity(this.platform, ZERO, ZERO);
      return;
    }
    const target = profileHeight(move, t + dt);
    world.setVelocity(this.platform, { x: 0, y: (target - this.height) / dt, z: 0 }, ZERO);
  }

  /** Reads the platform height after a step and moves the deck frame with it. */
  syncHeight(world: PhysicsBackend, t: number): void {
    this.height = world.getPose(this.platform).position.y - this.base.y;
    this.deck.frame = {
      ...this.deck.frame,
      origin: { ...this.deck.frame.origin, y: this.base.y + this.height },
    };
    if (this.move !== undefined && t >= this.move.start + this.move.duration - 1e-9)
      this.move = undefined;
  }

  get busyStatus(): 'lifting' | 'lowering' | 'processing' | undefined {
    if (this.phase === 'processing') return 'processing';
    if (this.move === undefined) return undefined;
    return this.move.to > this.move.from ? 'lifting' : 'lowering';
  }

  /**
   * Adds `seconds` in the current phase. As in 2D, a lift station works in every phase but idle; a
   * process station only while processing. Discharging into a full next object counts as blocked.
   */
  accumulate(seconds: number): void {
    this.stats.total += seconds;
    if (this.phase === 'discharging' && this.deck.status === 'blocked')
      this.stats.blocked += seconds;
    else if (this.node.kind === 'process' ? this.phase === 'processing' : this.phase !== 'idle')
      this.stats.working += seconds;
  }

  statistics(): Record<string, number> {
    const { working, blocked, total, cycles, cycleSum, done, processingSum } = this.stats;
    const share = (value: number) => (total > 0 ? value / total : 0);
    const meanCycleTime = cycles > 0 ? cycleSum / cycles / US_PER_S : 0;
    const common = {
      utilization: share(working),
      blockedShare: share(blocked),
      idleShare: share(total - working - blocked),
    };
    if (this.node.kind === 'process') {
      return {
        ...common,
        palletsProcessed: done,
        meanProcessingTime: done > 0 ? processingSum / done / US_PER_S : 0,
        meanCycleTime,
      };
    }
    return { ...common, palletsMoved: done, meanCycleTime };
  }

  /** A cycle runs from accepting a pallet to being free again. */
  private endCycle(now: SimTime): void {
    this.phase = 'idle';
    this.stats.cycles += 1;
    this.stats.cycleSum += now - this.cycleStart;
  }

  /** The pallet is fully on the deck and at rest, held at the centre. */
  private centred(): boolean {
    const pallet = this.deck.pallets[0];
    if (pallet === undefined || !this.deck.holding || this.deck.speed > 0) return false;
    const e = this.deck.extent(pallet);
    return e.rear >= -ENTRY_TOLERANCE && Math.abs(e.speed) < REST_SPEED;
  }

  /** The first connected exit port (the elevator prefers OUT_NEAR, S14 §1), else the first one. */
  private exitPort(): LiftPort {
    const port = this.exitPorts.find((p) => this.exits.has(p.portId)) ?? this.exitPorts[0];
    if (port === undefined) throw new Error(`${this.id} has no exit port`);
    return port;
  }

  private discharge(): void {
    const port = this.exitPort();
    this.phase = 'discharging';
    this.deck.direction = port.atEnd ? 1 : -1;
    this.deck.end = this.deck.spec.length;
    this.deck.exit = this.exits.get(port.portId);
    this.deck.holding = false;
  }

  private startMove(to: number, t: number): void {
    const from = this.height;
    const distance = Math.abs(to - from);
    if (this.node.kind !== 'lift' || distance < 1e-6) return;
    const speed = to > from ? this.node.upSpeed : this.node.downSpeed;
    const accel = this.node.verticalAccel;
    const duration =
      distance >= (speed * speed) / accel
        ? distance / speed + speed / accel
        : 2 * Math.sqrt(distance / accel);
    this.move = { from, to, start: t, duration, speed, accel };
  }

  private drawProcessingTime(): SimTime {
    if (this.node.kind !== 'process') return 0;
    const { processingMode, processingTime: mean, processingStdDev } = this.node;
    let seconds = mean;
    if (this.rng !== undefined && processingMode === 'normal') {
      seconds =
        mean +
        processingStdDev *
          Math.sqrt(-2 * Math.log(1 - this.rng.next())) *
          Math.cos(2 * Math.PI * this.rng.next());
    } else if (this.rng !== undefined && processingMode === 'exponential') {
      seconds = -mean * Math.log(1 - this.rng.next());
    }
    return secondsToSimTime(Math.max(0, seconds));
  }
}

/** Height along a trapezoidal move at time `t` (s), clamped to its ends (T04 §2). */
export function profileHeight(move: VerticalMove, t: number): number {
  const { from, to, start, duration, speed, accel } = move;
  const tau = Math.min(Math.max(t - start, 0), duration);
  const sign = Math.sign(to - from);
  const distance = Math.abs(to - from);
  const ramp = Math.min(speed / accel, duration / 2);
  const peak = accel * ramp;
  let travelled: number;
  if (tau <= ramp) travelled = 0.5 * accel * tau * tau;
  else if (tau <= duration - ramp) travelled = 0.5 * accel * ramp * ramp + peak * (tau - ramp);
  else travelled = distance - 0.5 * accel * (duration - tau) ** 2;
  return from + sign * Math.min(distance, travelled);
}
