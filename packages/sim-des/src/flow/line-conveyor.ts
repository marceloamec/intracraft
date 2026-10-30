// 2D behaviour of a straight conveyor without accumulation (S01 §3 "none", S02): all pallets on it
// share one motion. While a pallet crosses to the next conveyor both move together (flow-simulation.ts).
import type { SimTime } from '@intracraft/core-model';
import type { LineNode } from './network';
import type { Pallet } from './pallet';

/** Position tolerance (m) for "has reached". */
export const EPS = 1e-6;
/** Speed tolerance (m/s) for "has reached the target speed". */
export const SPEED_EPS = 1e-6;

/**
 * empty: no pallets; working: carrying and allowed to run; blocked: braking or held at the end;
 * stopped: carrying but switched off by the controller (`run` = false).
 */
export type LineStatus = 'empty' | 'working' | 'blocked' | 'stopped';

export class LineConveyor {
  /** Carried pallets, front first. */
  readonly pallets: Pallet[] = [];
  speed = 0;
  accel = 0;
  /** Time (s) at which `speed` and the pallet fronts are valid. */
  t0 = 0;
  /** Pallet already accepted by the next object and crossing to it. */
  outgoing: Pallet | undefined;
  /** Pallet accepted from the previous conveyor and crossing onto this one. */
  incoming: Pallet | undefined;
  /** The front pallet must stop at the end because the next object cannot take it. */
  holding = false;
  /** Where a held front pallet stops (m): the conveyor end, or the centre of a turntable deck. */
  end: number;
  status: LineStatus = 'empty';
  readonly stats = { empty: 0, working: 0, blocked: 0, stopped: 0, occupancy: 0, processed: 0, transitSum: 0 };

  /**
   * @param carrier Instance the pallets are drawn on: the conveyor itself, or the station a deck
   *   belongs to.
   */
  constructor(
    readonly node: LineNode,
    readonly carrier: string = node.id,
  ) {
    this.end = node.length;
  }

  get front(): Pallet | undefined {
    return this.pallets[0];
  }

  /** Tail position of the last pallet, or undefined when empty. */
  get rearTail(): number | undefined {
    const rear = this.pallets.at(-1);
    return rear === undefined ? undefined : rear.front - rear.length;
  }

  /** Moves the pallets along the current motion segment up to time `t` (s). */
  advanceTo(t: number): void {
    const dt = t - this.t0;
    if (dt <= 0) return;
    const moved = travel(this.speed, this.accel, dt);
    for (const pallet of this.pallets) pallet.front += moved;
    this.speed = Math.max(0, this.speed + this.accel * dt);
    this.t0 = t;
  }

  /** Front of a carried pallet at time `t` (s), without changing the state. */
  frontAt(pallet: Pallet, t: number): number {
    return pallet.front + travel(this.speed, this.accel, t - this.t0);
  }

  speedAt(t: number): number {
    return Math.max(0, this.speed + this.accel * Math.max(0, t - this.t0));
  }

  /** Adds `seconds` spent in the current status to the statistics. */
  accumulate(seconds: number): void {
    this.stats[this.status] += seconds;
    this.stats.occupancy += this.pallets.length * seconds;
  }

  /** The front pallet's tail cleared the end: it is no longer carried by this conveyor. */
  release(now: SimTime): Pallet {
    const pallet = this.pallets.shift();
    if (pallet === undefined || pallet !== this.outgoing) throw new Error(`${this.node.id}: no outgoing pallet to release`);
    this.outgoing = undefined;
    this.stats.processed += 1;
    this.stats.transitSum += now - pallet.enteredAt;
    return pallet;
  }
}

/** Distance covered in `dt` seconds from `speed` with constant `accel`, never moving backwards. */
function travel(speed: number, accel: number, dt: number): number {
  if (dt <= 0) return 0;
  const moving = accel < 0 ? Math.min(dt, speed / -accel) : dt;
  return speed * moving + 0.5 * accel * moving * moving;
}
