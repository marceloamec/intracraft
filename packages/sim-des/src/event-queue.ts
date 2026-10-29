// Future event list as a binary min-heap ordered by (time, priority, sequence).
// The sequence number makes ties deterministic: equal time and priority → insertion order.
import type { SimTime } from '@intracraft/core-model';

export interface QueuedEvent<T> {
  readonly time: SimTime;
  readonly priority: number;
  readonly seq: number;
  readonly payload: T;
}

export class EventQueue<T> {
  private readonly heap: QueuedEvent<T>[] = [];
  private nextSeq = 0;

  get size(): number {
    return this.heap.length;
  }

  push(time: SimTime, priority: number, payload: T): QueuedEvent<T> {
    if (!Number.isInteger(time)) throw new Error(`Event time must be an integer (µs), got ${time}`);
    const event: QueuedEvent<T> = { time, priority, seq: this.nextSeq++, payload };
    this.heap.push(event);
    this.siftUp(this.heap.length - 1);
    return event;
  }

  peek(): QueuedEvent<T> | undefined {
    return this.heap[0];
  }

  pop(): QueuedEvent<T> | undefined {
    const top = this.heap[0];
    const last = this.heap.pop();
    if (top === undefined || last === undefined) return undefined;
    if (this.heap.length > 0) {
      this.heap[0] = last;
      this.siftDown(0);
    }
    return top;
  }

  private siftUp(index: number): void {
    let i = index;
    while (i > 0) {
      const parent = (i - 1) >> 1;
      if (!this.before(i, parent)) break;
      this.swap(i, parent);
      i = parent;
    }
  }

  private siftDown(index: number): void {
    let i = index;
    for (;;) {
      const left = 2 * i + 1;
      const right = left + 1;
      let smallest = i;
      if (left < this.heap.length && this.before(left, smallest)) smallest = left;
      if (right < this.heap.length && this.before(right, smallest)) smallest = right;
      if (smallest === i) return;
      this.swap(i, smallest);
      i = smallest;
    }
  }

  private before(a: number, b: number): boolean {
    return compareEvents(this.at(a), this.at(b)) < 0;
  }

  private swap(a: number, b: number): void {
    const tmp = this.at(a);
    this.heap[a] = this.at(b);
    this.heap[b] = tmp;
  }

  private at(index: number): QueuedEvent<T> {
    const event = this.heap[index];
    if (event === undefined) throw new Error(`Heap index out of range: ${index}`);
    return event;
  }
}

export function compareEvents<T>(a: QueuedEvent<T>, b: QueuedEvent<T>): number {
  return a.time - b.time || a.priority - b.priority || a.seq - b.seq;
}
