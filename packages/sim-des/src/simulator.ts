// Minimal next-event time-advance loop. Components schedule events; the clock jumps
// from one event to the next. Cancelled events are skipped lazily when popped.
import type { SimTime } from '@intracraft/core-model';
import { EventQueue } from './event-queue';

export interface SimEvent {
  readonly type: string;
  readonly target: string;
  readonly data?: unknown;
}

export type EventHandler = (sim: DesSimulator, event: SimEvent) => void;

export interface EventTrace {
  time: SimTime;
  type: string;
  target: string;
}

export class DesSimulator {
  private readonly queue = new EventQueue<SimEvent>();
  private readonly handlers = new Map<string, EventHandler>();
  private readonly cancelled = new Set<number>();
  private clock: SimTime = 0;
  private readonly keepTrace: boolean;
  readonly trace: EventTrace[] = [];

  /** `trace: false` skips recording every event (long interactive runs). */
  constructor(options: { trace?: boolean } = {}) {
    this.keepTrace = options.trace ?? true;
  }

  get now(): SimTime {
    return this.clock;
  }

  on(type: string, handler: EventHandler): void {
    this.handlers.set(type, handler);
  }

  /** Schedule an event `delay` µs from now. Returns a ticket usable with cancel(). */
  schedule(delay: SimTime, event: SimEvent, priority = 0): number {
    if (delay < 0) throw new Error(`Cannot schedule in the past (delay ${delay})`);
    return this.queue.push(this.clock + delay, priority, event).seq;
  }

  cancel(ticket: number): void {
    this.cancelled.add(ticket);
  }

  /** Time of the next pending (not cancelled) event, or undefined when nothing is scheduled. */
  nextEventTime(): SimTime | undefined {
    for (;;) {
      const next = this.queue.peek();
      if (next === undefined || !this.cancelled.has(next.seq)) return next?.time;
      this.queue.pop();
      this.cancelled.delete(next.seq);
    }
  }

  /** Process events until the queue is empty or the next event is after `endTime`. */
  runUntil(endTime: SimTime): void {
    for (;;) {
      const next = this.queue.peek();
      if (next === undefined || next.time > endTime) break;
      this.queue.pop();
      if (this.cancelled.delete(next.seq)) continue;
      this.clock = next.time;
      if (this.keepTrace) this.trace.push({ time: next.time, type: next.payload.type, target: next.payload.target });
      const handler = this.handlers.get(next.payload.type);
      if (handler === undefined) throw new Error(`No handler for event type "${next.payload.type}"`);
      handler(this, next.payload);
    }
    this.clock = Math.max(this.clock, endTime);
  }
}
