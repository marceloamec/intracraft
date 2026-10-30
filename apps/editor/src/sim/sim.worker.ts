// Simulation Web Worker: runs the 2D model off the UI thread and posts frames (A02 §3).
import type { FromWorker, ToWorker } from './protocol';
import { SimRunner } from './sim-runner';

const TICK_MS = 33;

const runner = new SimRunner(() => performance.now());
let last = performance.now();

self.onmessage = (event: MessageEvent<ToWorker>) => {
  respond(() => runner.handle(event.data));
  last = performance.now();
};

setInterval(() => {
  const now = performance.now();
  respond(() => runner.tick(now - last));
  last = now;
}, TICK_MS);

function respond(work: () => FromWorker | undefined): void {
  try {
    const message = work();
    if (message !== undefined) self.postMessage(message);
  } catch (error) {
    self.postMessage({ type: 'error', message: error instanceof Error ? error.message : String(error) } satisfies FromWorker);
  }
}
