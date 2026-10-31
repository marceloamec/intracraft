// 2D behaviour of the gravity roller line (S15, ADR-0014): pallets roll down one by one. From rest a
// pallet accelerates with a = g·(slope − Crr) up to the brake speed, then keeps it. It stops when it
// touches what is ahead: the pallet in front, the closed separator or the end stop (no braking before
// a contact). A pallet that touches a moving pallet goes on with it.
//   - Queue start: when a stopped pallet starts, the one stopped behind it starts `startDelay` later.
//   - Separator at x_S = length − pickupLength: open while the pick-up position is empty; it closes
//     once the pallet that entered the position has passed it, and opens `separatorTime` after the
//     operator has taken that pallet. The first waiting pallet then rolls to the end stop at once.
// Positions are the pallet fronts along the line (plan length, m), valid at time `t0` (s). The line
// predicts its own next change; FlowSimulation schedules it as `gravity.motion`.
import { GRAVITY, type SimTime, US_PER_S } from '@intracraft/core-model';
import { EPS, SPEED_EPS } from './line-conveyor';
import type { GravityNode } from './network';
import type { Pallet } from './pallet';

/** Times closer than this are the same instant (s). */
const TIME_EPS = 1e-9;

export type GravityStatus = 'empty' | 'working' | 'waiting' | 'blocked';

export interface GravityPallet {
  pallet: Pallet;
  speed: number;
  accel: number;
  stopped: boolean;
  /** Stopped at the closed separator: starts as soon as it opens. */
  atSeparator: boolean;
  /** Time (s) at which a stopped pallet starts if the pallet ahead has moved away. */
  startAt?: number;
}

/** What limits a pallet's front: the end stop, the closed separator or the tail of the pallet ahead. */
interface Limit {
  x: number;
  kind: 'end' | 'separator' | 'pallet';
}

export class GravityLine {
  /** Pallets on the line, front (lowest) first. */
  readonly pallets: GravityPallet[] = [];
  /** Pallets accepted from the feeder and still crossing onto the line. */
  incoming = 0;
  /** Room a waiting feeder needs behind the last pallet (m), while it waits. */
  waitingLength: number | undefined;
  /** Scheduled `gravity.motion` event. */
  ticket: number | undefined;
  /** a = g·(slope − Crr) from rest (m/s²); ≤ 0 means a pallet at rest does not start (S15 §4). */
  readonly accel: number;
  /** Separator position, measured like the pallet fronts (m). */
  readonly separatorX: number;
  readonly stats = { delivered: 0, maxQueue: 0, queueTime: 0, fullTime: 0, pickupWaitTime: 0, total: 0, transitSum: 0 };
  private t0 = 0;
  private separatorClosed = false;
  /** Pallet that entered the pick-up position and has not been taken yet. */
  private holder: GravityPallet | undefined;
  /** Time (s) at which the closed separator opens again. */
  private openAt: number | undefined;

  constructor(readonly node: GravityNode) {
    this.accel = GRAVITY * (node.slope - node.rollingResistance);
    this.separatorX = node.length - node.pickupLength;
  }

  /** A whole pallet of `length` fits behind the last one and nothing else is entering. */
  canAccept(length: number): boolean {
    if (this.incoming > 0) return false;
    const rear = this.pallets.at(-1);
    return rear === undefined || tail(rear) >= length - EPS;
  }

  /** The pallet's tail cleared the feeder: it is fully on the line, moving at min(feeder speed, brake speed). */
  receive(pallet: Pallet, feederSpeed: number, now: SimTime): void {
    const t = now / US_PER_S;
    this.advanceTo(t);
    this.incoming = Math.max(0, this.incoming - 1);
    pallet.front = pallet.length;
    pallet.enteredAt = now;
    const speed = Math.min(feederSpeed, this.node.brakeSpeed);
    const entering: GravityPallet = { pallet, speed, accel: 0, stopped: false, atSeparator: false };
    entering.accel = this.rollingAccel(entering);
    this.pallets.push(entering);
    this.stats.maxQueue = Math.max(this.stats.maxQueue, this.pallets.length);
    this.resolve(t);
  }

  /** The pallet at rest against the end stop, waiting for the operator. */
  get atEndStop(): GravityPallet | undefined {
    const first = this.pallets[0];
    return first !== undefined && first.stopped && first.pallet.front >= this.node.length - EPS ? first : undefined;
  }

  /** The operator takes the pallet at the end stop; the separator opens `separatorTime` later. */
  take(now: SimTime): Pallet {
    const first = this.atEndStop;
    if (first === undefined) throw new Error(`${this.node.id}: no pallet at the end stop`);
    this.pallets.shift();
    this.stats.delivered += 1;
    this.stats.transitSum += now - first.pallet.enteredAt;
    if (this.holder === first) this.holder = undefined;
    const t = now / US_PER_S;
    this.openAt = t + this.node.separatorTime;
    this.resolve(t);
    return first.pallet;
  }

  /** Moves every pallet to time `t` (s) and settles contacts, starts and the separator at `t`. */
  advanceTo(t: number): void {
    const dt = t - this.t0;
    if (dt > 0) {
      for (const p of this.pallets) {
        if (p.stopped) continue;
        const moving = p.accel < 0 ? Math.min(dt, p.speed / -p.accel) : dt;
        p.pallet.front += p.speed * moving + 0.5 * p.accel * moving * moving;
        p.speed = Math.max(0, p.speed + p.accel * moving);
      }
      this.t0 = t;
    }
    this.resolve(t);
  }

  /** Front of a pallet at time `t` (s) without changing the state. */
  frontAt(p: GravityPallet, t: number): number {
    if (p.stopped) return p.pallet.front;
    const dt = Math.max(0, t - this.t0);
    const moving = p.accel < 0 ? Math.min(dt, p.speed / -p.accel) : dt;
    return p.pallet.front + p.speed * moving + 0.5 * p.accel * moving * moving;
  }

  speedAt(p: GravityPallet, t: number): number {
    if (p.stopped) return 0;
    return Math.max(0, Math.min(this.node.brakeSpeed, p.speed + p.accel * Math.max(0, t - this.t0)));
  }

  /** Earliest predicted change (s), or undefined when nothing will move. */
  nextChange(): number | undefined {
    const times: number[] = [];
    if (this.openAt !== undefined) times.push(this.openAt);
    for (const [i, p] of this.pallets.entries()) {
      if (p.stopped) {
        if (p.startAt !== undefined) times.push(p.startAt);
        continue;
      }
      const ahead = this.pallets[i - 1];
      if (p.accel > 0) times.push(this.t0 + (this.node.brakeSpeed - p.speed) / p.accel);
      if (p.accel < 0) times.push(this.t0 + p.speed / -p.accel);
      const limit = this.limit(p, ahead);
      const gap = limit.x - p.pallet.front;
      const contact =
        limit.kind === 'pallet' && ahead !== undefined && !ahead.stopped
          ? firstRoot(gap, ahead.speed - p.speed, ahead.accel - p.accel)
          : reach(p, gap);
      if (contact !== undefined) times.push(this.t0 + contact);
      if (this.holder === undefined && !this.separatorClosed && p.pallet.front < this.separatorX - EPS) {
        const passing = reach(p, this.separatorX - p.pallet.front);
        if (passing !== undefined) times.push(this.t0 + passing);
      }
      if (this.holder === p && !this.separatorClosed) {
        const cleared = reach(p, this.separatorX - tail(p));
        if (cleared !== undefined) times.push(this.t0 + cleared);
      }
    }
    const rear = this.pallets.at(-1);
    if (this.waitingLength !== undefined && rear !== undefined && !rear.stopped && tail(rear) < this.waitingLength - EPS) {
      const room = reach(rear, this.waitingLength - tail(rear));
      if (room !== undefined) times.push(this.t0 + room);
    }
    const future = times.filter((time) => time > this.t0 + TIME_EPS);
    return future.length === 0 ? undefined : Math.min(...future);
  }

  get status(): GravityStatus {
    if (this.pallets.length === 0 && this.incoming === 0) return 'empty';
    if (this.waitingLength !== undefined) return 'blocked';
    return this.pallets.some((p) => !p.stopped) ? 'working' : 'waiting';
  }

  accumulate(seconds: number): void {
    this.stats.total += seconds;
    this.stats.queueTime += this.pallets.length * seconds;
    if (this.waitingLength !== undefined) this.stats.fullTime += seconds;
    if (this.atEndStop !== undefined) this.stats.pickupWaitTime += seconds;
  }

  /** Statistics keyed like the definition's `statistics` list (S15 §3). */
  statistics(): Record<string, number> {
    const { delivered, maxQueue, queueTime, fullTime, pickupWaitTime, total, transitSum } = this.stats;
    const share = (seconds: number) => (total > 0 ? seconds / total : 0);
    return {
      palletsDelivered: delivered,
      meanQueue: share(queueTime),
      maxQueue,
      fullShare: share(fullTime),
      pickupWaitShare: share(pickupWaitTime),
      meanTransitTime: delivered > 0 ? transitSum / delivered / US_PER_S : 0,
    };
  }

  /** Settles the line at time `t`: separator, delayed starts, brake speed, contacts, front to back. */
  private resolve(t: number): void {
    if (this.openAt !== undefined && this.openAt <= t + TIME_EPS) {
      this.openAt = undefined;
      this.separatorClosed = false;
    }
    for (const [i, p] of this.pallets.entries()) {
      const ahead = this.pallets[i - 1];
      if (p.stopped) this.tryStart(p, i, ahead, t);
      if (!p.stopped) {
        if (p.accel > 0 && p.speed >= this.node.brakeSpeed - SPEED_EPS) {
          p.speed = this.node.brakeSpeed;
          p.accel = 0;
        }
        const limit = this.limit(p, ahead);
        if (p.pallet.front >= limit.x - EPS) {
          p.pallet.front = limit.x;
          if (limit.kind === 'pallet' && ahead !== undefined && !ahead.stopped) {
            // Touching a moving pallet: carry on with it.
            p.speed = ahead.speed;
            p.accel = ahead.accel;
          } else {
            this.stop(p, limit);
          }
        } else if (p.accel <= 0 && p.speed <= SPEED_EPS) {
          this.stop(p, limit);
        }
      }
      if (this.holder === undefined && !this.separatorClosed && p.pallet.front >= this.separatorX - EPS) this.holder = p;
      if (this.holder === p && !this.separatorClosed && tail(p) >= this.separatorX - EPS) this.separatorClosed = true;
    }
  }

  /** Starts a stopped pallet whose way is free: at once after the separator, else at its `startAt`. */
  private tryStart(p: GravityPallet, index: number, ahead: GravityPallet | undefined, t: number): void {
    const limit = this.limit(p, ahead);
    // Touching a pallet that has started does not block: the two move off together.
    const leaving = limit.kind === 'pallet' && ahead !== undefined && !ahead.stopped;
    if (limit.x - p.pallet.front <= EPS && !leaving) {
      // Still blocked: a pending start waits for the pallet ahead to move again.
      if (limit.kind === 'pallet' && ahead?.stopped === true) delete p.startAt;
      return;
    }
    const due = p.atSeparator || (p.startAt !== undefined && p.startAt <= t + TIME_EPS);
    if (!due) {
      p.startAt ??= t + this.node.startDelay;
      if (p.startAt > t + TIME_EPS) return;
    }
    if (this.accel <= 0) return;
    p.stopped = false;
    p.atSeparator = false;
    delete p.startAt;
    p.speed = 0;
    p.accel = this.accel;
    // The pallet stopped behind this one follows `startDelay` later.
    const behind = this.pallets[index + 1];
    if (behind?.stopped === true && behind.pallet.front >= tail(p) - EPS) behind.startAt = t + this.node.startDelay;
  }

  private stop(p: GravityPallet, limit: Limit): void {
    p.stopped = true;
    p.speed = 0;
    p.accel = 0;
    p.atSeparator = limit.kind === 'separator';
    delete p.startAt;
  }

  /** Acceleration of a rolling pallet: up to the brake speed, then none. */
  private rollingAccel(p: GravityPallet): number {
    return p.speed < this.node.brakeSpeed - SPEED_EPS ? this.accel : 0;
  }

  private limit(p: GravityPallet, ahead: GravityPallet | undefined): Limit {
    let limit: Limit = { x: this.node.length, kind: 'end' };
    if (this.separatorClosed && p.pallet.front <= this.separatorX + EPS && this.separatorX < limit.x) {
      limit = { x: this.separatorX, kind: 'separator' };
    }
    if (ahead !== undefined && tail(ahead) < limit.x) limit = { x: tail(ahead), kind: 'pallet' };
    return limit;
  }
}

function tail(p: GravityPallet): number {
  return p.pallet.front - p.pallet.length;
}

/** Time (s) for a pallet to cover `distance` with its speed and acceleration, or undefined. */
function reach(p: GravityPallet, distance: number): number | undefined {
  if (distance <= 0) return undefined;
  const root = firstRoot(-distance, p.speed, p.accel);
  // A decelerating pallet stops before covering the distance.
  if (root !== undefined && p.accel < 0 && root > p.speed / -p.accel) return undefined;
  return root;
}

/** Smallest t > 0 with c + b·t + ½·a·t² = 0 (c: gap, b: relative speed, a: relative acceleration). */
function firstRoot(c: number, b: number, a: number): number | undefined {
  if (Math.abs(a) < 1e-12) {
    if (Math.abs(b) < 1e-12) return undefined;
    const t = -c / b;
    return t > TIME_EPS ? t : undefined;
  }
  const disc = b * b - 2 * a * c;
  if (disc < 0) return undefined;
  const root = Math.sqrt(disc);
  const roots = [(-b - root) / a, (-b + root) / a].filter((t) => t > TIME_EPS);
  return roots.length === 0 ? undefined : Math.min(...roots);
}
