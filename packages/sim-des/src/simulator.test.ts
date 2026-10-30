import { describe, expect, it } from 'vitest';
import { EventQueue } from './event-queue';
import { DesSimulator } from './simulator';

describe('event queue ordering', () => {
  it('orders by time, then priority, then insertion sequence', () => {
    const queue = new EventQueue<string>();
    queue.push(200, 0, 'late');
    queue.push(100, 1, 'low-priority');
    queue.push(100, 0, 'first-tie');
    queue.push(100, 0, 'second-tie');
    const order = [queue.pop(), queue.pop(), queue.pop(), queue.pop()].map((e) => e?.payload);
    expect(order).toEqual(['first-tie', 'second-tie', 'low-priority', 'late']);
  });

  it('rejects non-integer times', () => {
    expect(() => new EventQueue<string>().push(1.5, 0, 'x')).toThrow();
  });
});

describe('DesSimulator', () => {
  it('jumps the clock from event to event', () => {
    const sim = new DesSimulator();
    const seen: number[] = [];
    sim.on('tick', (s) => seen.push(s.now));
    sim.schedule(5_000_000, { type: 'tick', target: 'a' });
    sim.schedule(1_000_000, { type: 'tick', target: 'b' });
    sim.runUntil(10_000_000);
    expect(seen).toEqual([1_000_000, 5_000_000]);
    expect(sim.now).toBe(10_000_000);
  });

  it('skips cancelled events (stale predictions)', () => {
    const sim = new DesSimulator();
    const seen: string[] = [];
    sim.on('arrive', (_s, e) => seen.push(e.target));
    const stale = sim.schedule(1_000, { type: 'arrive', target: 'stale' });
    sim.schedule(2_000, { type: 'arrive', target: 'fresh' });
    sim.cancel(stale);
    sim.runUntil(10_000);
    expect(seen).toEqual(['fresh']);
  });

  it('reports the next pending event time, skipping cancelled ones', () => {
    const sim = new DesSimulator();
    sim.on('x', () => undefined);
    const first = sim.schedule(1_000, { type: 'x', target: 'a' });
    sim.schedule(3_000, { type: 'x', target: 'b' });
    sim.cancel(first);
    expect(sim.nextEventTime()).toBe(3_000);
    sim.runUntil(3_000);
    expect(sim.nextEventTime()).toBeUndefined();
  });

  it('is deterministic: the same schedule gives the same trace', () => {
    expect(runChain()).toEqual(runChain());
  });
});

function runChain() {
  const sim = new DesSimulator();
  sim.on('step', (s, e) => {
    if (s.now < 1_000_000) s.schedule(250_000, { type: 'step', target: e.target });
  });
  sim.schedule(0, { type: 'step', target: 'p1' });
  sim.schedule(0, { type: 'step', target: 'p2' });
  sim.runUntil(2_000_000);
  return sim.trace;
}
