// 3D behaviour of the gravity roller line (S15): pallets roll down free rollers on the slope, brake
// rollers cap their speed, a separator holds the next pallet back from the pick-up position and an
// end stop takes the impact. The rules are those of the 2D model (ADR-0014), measured on the bodies.
//
// Free rollers are kinematic cylinders that follow the pallet above them: each step they turn at the
// speed the pallet reaches rolling freely, v + g·(s − Crr)·dt, capped at the brake speed. Contacts
// with the separator, the end stop and the other pallets stay physical (ADR-0016).
import { GRAVITY, type SimTime, type Vec3 } from '@intracraft/core-model';
import type { GravityNode } from '@intracraft/sim-des';
import type { BodyHandle, PhysicsBackend } from '../backend';
import type { SinkEnd } from './deck';
import { type DeckFrame, frameAxis, scale, toLocal } from './frame';
import { type Extent, type WorldPallet, extentOn } from './pallet-body';

/** A pallet this close to what is ahead touches it (m). */
const TOUCH = 0.01;
/** Below this speed a touching pallet counts as stopped (m/s). */
const STOP_SPEED = 0.01;
/** Local height of the separator plate centre when closed and when open (m). */
export const SEPARATOR_CLOSED_Y = 0.005;
export const SEPARATOR_OPEN_Y = -0.085;
const SEPARATOR_SPEED = 0.5;
/** Thickness of the separator plate along travel; its upstream face is at the separator position (m). */
export const SEPARATOR_THICKNESS = 0.02;
const ZERO: Vec3 = { x: 0, y: 0, z: 0 };

interface GravityState {
  stopped: boolean;
  /** Stopped at the closed separator: starts as soon as it opens. */
  atSeparator: boolean;
  /** Delayed start behind a pallet that started (s). */
  startAt?: number;
}

export interface GravityRoller {
  body: BodyHandle;
  /** Position along the line (m from the entry). */
  s: number;
}

export type GravityStatus = 'empty' | 'working' | 'waiting' | 'blocked';

export class GravityLine3D {
  /** Pallets on the line, front first. */
  readonly pallets: WorldPallet[] = [];
  /** Pallets accepted and still pushed in by the feeder; the free rollers already carry their front. */
  readonly entering: WorldPallet[] = [];
  /** The operator at OUT. */
  sink: SinkEnd | undefined;
  /** Position of the separator (m from the entry): the start of the pick-up position. */
  readonly separatorX: number;
  readonly accel: number;
  status: GravityStatus = 'empty';
  /** Length of the pallet a feeder is waiting to push in while the line is full, as in 2D. */
  waitingLength: number | undefined;
  readonly stats = {
    delivered: 0,
    maxQueue: 0,
    queue: 0,
    full: 0,
    pickupWait: 0,
    total: 0,
    transitSum: 0,
  };
  private readonly state = new Map<number, GravityState>();
  private separatorClosed = false;
  private holder: WorldPallet | undefined;
  /** Time at which the closed separator opens again (s). */
  private openAt: number | undefined;

  constructor(
    readonly node: GravityNode,
    readonly frame: DeckFrame,
    private readonly rollers: GravityRoller[],
    private readonly diameter: number,
    private readonly separator: BodyHandle,
  ) {
    this.separatorX = node.length - node.pickupLength;
    this.accel = GRAVITY * (node.slope - node.rollingResistance);
  }

  get rollerBodies(): BodyHandle[] {
    return this.rollers.map((roller) => roller.body);
  }

  extent(pallet: WorldPallet): Extent {
    return extentOn(pallet, this.frame, this.node.length, 1);
  }

  /**
   * A whole pallet of `length` fits behind the last one and nothing else is entering. Unlike 2D the
   * feeder's `gap` counts: the pallet is only free of the feeder once its front is that far in.
   */
  canAccept(length: number): boolean {
    if (this.entering.length > 0) return false;
    const last = this.pallets.at(-1);
    return last === undefined || this.extent(last).rear >= length;
  }

  /** The pallet's tail cleared the feeder: from now on it rolls on the line. */
  receive(pallet: WorldPallet, now: SimTime): void {
    const index = this.entering.indexOf(pallet);
    if (index >= 0) this.entering.splice(index, 1);
    pallet.carrier = this.node.id;
    pallet.enteredAt = now;
    this.pallets.push(pallet);
    this.state.set(pallet.id, { stopped: false, atSeparator: false });
    this.stats.maxQueue = Math.max(this.stats.maxQueue, this.pallets.length);
  }

  /** The pallet at rest against the end stop, waiting for the operator. */
  get atEndStop(): WorldPallet | undefined {
    const first = this.pallets[0];
    if (first === undefined || this.state.get(first.id)?.stopped !== true) return undefined;
    return this.extent(first).front >= this.node.length - TOUCH ? first : undefined;
  }

  /** The operator takes the pallet at the end stop; the separator opens `separatorTime` later. */
  take(now: SimTime, t: number): WorldPallet {
    const first = this.atEndStop;
    if (first === undefined) throw new Error(`${this.node.id}: no pallet at the end stop`);
    this.pallets.shift();
    this.state.delete(first.id);
    this.stats.delivered += 1;
    this.stats.transitSum += now - first.enteredAt;
    if (this.holder === first) this.holder = undefined;
    this.openAt = t + this.node.separatorTime;
    return first;
  }

  /** Stops, delayed starts and the separator at time `t` (s), from the measured positions. */
  update(t: number): void {
    if (this.openAt !== undefined && this.openAt <= t) {
      this.openAt = undefined;
      this.separatorClosed = false;
    }
    const extents = this.pallets.map((p) => this.extent(p));
    let moving = false;
    for (const [i, pallet] of this.pallets.entries()) {
      const e = extents[i];
      const st = this.state.get(pallet.id);
      if (e === undefined || st === undefined) continue;
      const ahead = this.pallets[i - 1];
      const aheadState = ahead === undefined ? undefined : this.state.get(ahead.id);
      const aheadRear = extents[i - 1]?.rear;
      let limit = this.node.length;
      let kind: 'end' | 'separator' | 'pallet' = 'end';
      if (this.separatorClosed && e.front <= this.separatorX + TOUCH && this.separatorX < limit) {
        [limit, kind] = [this.separatorX, 'separator'];
      }
      if (aheadRear !== undefined && aheadRear < limit) [limit, kind] = [aheadRear, 'pallet'];
      const touching = limit - e.front <= TOUCH;
      // Touching a pallet that has started does not block: the two move off together.
      const leaving = kind === 'pallet' && aheadState?.stopped === false;

      if (!st.stopped) {
        if (e.rear >= 0 && touching && !leaving && Math.abs(e.speed) < STOP_SPEED) {
          st.stopped = true;
          st.atSeparator = kind === 'separator';
          delete st.startAt;
        }
      } else if (touching && !leaving) {
        if (kind === 'pallet' && aheadState?.stopped === true) delete st.startAt;
      } else {
        const due =
          (st.atSeparator && !this.separatorClosed) ||
          (st.startAt !== undefined && st.startAt <= t);
        if (!due) st.startAt ??= t + this.node.startDelay;
        if (due || (st.startAt !== undefined && st.startAt <= t)) this.start(pallet, st, i, t);
      }
      if (!st.stopped) moving = true;

      if (this.holder === undefined && !this.separatorClosed && e.front >= this.separatorX)
        this.holder = pallet;
      if (
        this.holder === pallet &&
        !this.separatorClosed &&
        e.rear >= this.separatorX + SEPARATOR_THICKNESS + TOUCH &&
        !this.overSeparator()
      ) {
        this.separatorClosed = true;
      }
    }
    if (this.pallets.length === 0 && this.entering.length === 0) this.status = 'empty';
    else if (this.waitingLength !== undefined) this.status = 'blocked';
    else this.status = moving || this.entering.length > 0 ? 'working' : 'waiting';
  }

  /** Free rollers follow their pallet; the separator plate moves toward its position. */
  drive(world: PhysicsBackend, dt: number): void {
    const targets = [...this.pallets, ...this.entering].map((pallet) => {
      const e = this.extent(pallet);
      const st = this.state.get(pallet.id);
      let speed: number;
      if (e.rear < 0)
        speed = Math.max(0, e.speed); // still pushed by the feeder: rollers turn freely
      else if (st?.stopped !== false) speed = 0;
      else speed = Math.min(this.node.brakeSpeed, Math.max(0, e.speed + this.accel * dt));
      return { e, speed };
    });
    const axis = frameAxis(this.frame, 'z');
    for (const roller of this.rollers) {
      const over = targets.find(({ e }) => e.rear <= roller.s && roller.s <= e.front);
      const omega = (over?.speed ?? 0) / (this.diameter / 2);
      world.setVelocity(roller.body, ZERO, scale(axis, -omega));
    }
    const current = toLocal(this.frame, world.getPose(this.separator).position).y;
    const target = this.separatorClosed ? SEPARATOR_CLOSED_Y : SEPARATOR_OPEN_Y;
    const speed = Math.max(-SEPARATOR_SPEED, Math.min(SEPARATOR_SPEED, (target - current) / dt));
    world.setVelocity(this.separator, scale(frameAxis(this.frame, 'y'), speed), ZERO);
  }

  accumulate(seconds: number): void {
    this.stats.total += seconds;
    this.stats.queue += this.pallets.length * seconds;
    if (this.waitingLength !== undefined) this.stats.full += seconds;
    if (this.atEndStop !== undefined) this.stats.pickupWait += seconds;
  }

  statistics(): Record<string, number> {
    const { delivered, maxQueue, queue, full, pickupWait, total, transitSum } = this.stats;
    const share = (value: number) => (total > 0 ? value / total : 0);
    return {
      palletsDelivered: delivered,
      meanQueue: share(queue),
      maxQueue,
      fullShare: share(full),
      pickupWaitShare: share(pickupWait),
      meanTransitTime: delivered > 0 ? transitSum / delivered / 1e6 : 0,
    };
  }

  private start(pallet: WorldPallet, st: GravityState, index: number, t: number): void {
    st.stopped = false;
    st.atSeparator = false;
    delete st.startAt;
    // The pallet stopped behind this one follows `startDelay` later.
    const behind = this.pallets[index + 1];
    const behindState = behind === undefined ? undefined : this.state.get(behind.id);
    if (behind !== undefined && behindState?.stopped === true) {
      const gap = this.extent(pallet).rear - this.extent(behind).front;
      if (gap <= TOUCH) behindState.startAt = t + this.node.startDelay;
    }
  }

  /** Some pallet lies over the separator plate, so it cannot rise. */
  private overSeparator(): boolean {
    return this.pallets.some((p) => {
      const e = this.extent(p);
      return (
        e.rear < this.separatorX + SEPARATOR_THICKNESS + TOUCH && e.front > this.separatorX - TOUCH
      );
    });
  }
}
