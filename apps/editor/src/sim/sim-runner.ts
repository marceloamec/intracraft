// Drives a 2D flow simulation against the wall clock: play, pause, step to the next event and
// speed (×1, ×10, ×100, max). Free of worker APIs so it can be tested directly.
import { createBuiltInRegistry } from '@intracraft/library';
import { FlowSimulation, buildFlowNetwork } from '@intracraft/sim-des';
import type { SimFrame, SimSpeed, ToWorker } from './protocol';

/** Real milliseconds the "max" speed may spend per tick. */
export const MAX_SPEED_BUDGET_MS = 25;
/** Simulated time advanced per chunk at "max" speed (µs). */
const MAX_SPEED_CHUNK = 60_000_000;
/** At most one simulated hour per tick at "max" speed, so an idle model does not race ahead. */
export const MAX_SPEED_CHUNKS_PER_TICK = 60;
/** Statistics are attached to every n-th frame. */
const STATISTICS_EVERY = 8;

const registry = createBuiltInRegistry();

export class SimRunner {
  private sim: FlowSimulation | undefined;
  private playing = false;
  private speed: SimSpeed = 1;
  private frames = 0;
  private reportedIssues = 0;

  constructor(private readonly clock: () => number) {}

  /** Applies a command and returns the frame to show, if any. */
  handle(message: ToWorker): SimFrame | undefined {
    switch (message.type) {
      case 'load':
        this.sim = new FlowSimulation(buildFlowNetwork(message.layout, registry), { seed: message.seed });
        this.playing = false;
        this.reportedIssues = 0;
        return this.frame(true);
      case 'play':
        this.playing = true;
        return this.frame(false);
      case 'pause':
        this.playing = false;
        return this.frame(true);
      case 'setSpeed':
        this.speed = message.speed;
        return undefined;
      case 'step': {
        this.playing = false;
        const next = this.sim?.nextEventTime();
        if (next !== undefined) this.sim?.runUntil(next);
        return this.frame(true);
      }
    }
  }

  /** Advances the simulation by `realMs` of wall-clock time; returns the frame while playing. */
  tick(realMs: number): SimFrame | undefined {
    const sim = this.sim;
    if (sim === undefined || !this.playing) return undefined;
    if (this.speed === 'max') {
      const start = this.clock();
      let chunks = 0;
      do sim.runUntil(sim.now + MAX_SPEED_CHUNK);
      while (++chunks < MAX_SPEED_CHUNKS_PER_TICK && this.clock() - start < MAX_SPEED_BUDGET_MS);
    } else {
      sim.runUntil(sim.now + Math.round(realMs * 1000 * this.speed));
    }
    return this.frame(false);
  }

  private frame(withStatistics: boolean): SimFrame | undefined {
    const sim = this.sim;
    if (sim === undefined) return undefined;
    this.frames += 1;
    const frame: SimFrame = { type: 'frame', playing: this.playing, snapshot: sim.snapshot() };
    if (withStatistics || this.frames % STATISTICS_EVERY === 0) frame.statistics = sim.statistics();
    if (sim.issues.length !== this.reportedIssues) {
      frame.issues = [...sim.issues];
      this.reportedIssues = sim.issues.length;
    }
    return frame;
  }
}
