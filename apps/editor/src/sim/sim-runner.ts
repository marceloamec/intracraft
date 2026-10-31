// Drives a simulation against the wall clock: play, pause, step to the next event and speed
// (×1, ×10, ×100, max). The 2D run is created at once; the 3D run loads the physics engine first and
// reports its first frame through `post`. Free of worker APIs so it can be tested directly.
import {
  type Actor,
  SPAWN_OBJECT,
  type SimCommandRecord,
  parseSpawnObject,
} from '@intracraft/commands';
import type { Layout, SimTime } from '@intracraft/core-model';
import { createBuiltInRegistry } from '@intracraft/library';
import {
  type FlowIssue,
  FlowSimulation,
  type FlowStatistics,
  buildFlowNetwork,
} from '@intracraft/sim-des';
import { PhysicsFlowSimulation } from '@intracraft/sim-physics';
import type { FromWorker, RunMode, SimFrame, SimSnapshot, SimSpeed, ToWorker } from './protocol';

/** Real milliseconds the "max" speed may spend per tick. */
export const MAX_SPEED_BUDGET_MS = 25;
/** Simulated time advanced per chunk at "max" speed (µs). */
const MAX_SPEED_CHUNK = 60_000_000;
/** At most one simulated hour per tick at "max" speed, so an idle model does not race ahead. */
export const MAX_SPEED_CHUNKS_PER_TICK = 60;
/** The 3D run advances in chunks of this much simulated time (µs)… */
const PHYSICS_CHUNK = 50_000;
/** …while it stays within this real time per tick (ms); beyond it the run falls behind the clock. */
export const PHYSICS_BUDGET_MS = 25;
/** Statistics are attached to every n-th frame. */
const STATISTICS_EVERY = 8;

const registry = createBuiltInRegistry();

/** What the runner needs from a run: the 2D and the 3D simulations both fit. */
interface Run {
  readonly now: SimTime;
  readonly issues: FlowIssue[];
  runUntil(time: SimTime): void;
  nextEventTime(): SimTime | undefined;
  snapshot(): SimSnapshot;
  statistics(): FlowStatistics;
  dispose?(): void;
}

export class SimRunner {
  /** Simulation commands applied to the current run, in order (cleared by every load). */
  readonly log: SimCommandRecord[] = [];
  private sim: Run | undefined;
  private physics: PhysicsFlowSimulation | undefined;
  private mode: RunMode = 'des';
  private playing = false;
  private speed: SimSpeed = 1;
  private frames = 0;
  private reportedIssues = 0;
  /** Increases with every load, so a slow 3D load that was replaced is dropped. */
  private loads = 0;

  constructor(
    private readonly clock: () => number,
    /** Delivers frames that are ready later than the command (the 3D load). */
    private readonly post: (message: FromWorker) => void = () => {},
  ) {}

  /** Applies a command and returns the frame to show, if any. */
  handle(message: ToWorker): SimFrame | undefined {
    switch (message.type) {
      case 'load':
        return this.load(message.layout, message.seed, message.mode ?? 'des');
      case 'play':
        this.playing = true;
        return this.frame(false);
      case 'pause':
        this.playing = false;
        return this.frame(true);
      case 'setSpeed':
        this.speed = message.speed;
        return undefined;
      case 'spawn':
        return this.spawn(message.input, message.actor ?? 'user');
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
    const start = this.clock();
    if (this.mode === 'physics') {
      const target =
        this.speed === 'max' ? Infinity : sim.now + Math.round(realMs * 1000 * this.speed);
      while (sim.now < target && this.clock() - start < PHYSICS_BUDGET_MS) {
        sim.runUntil(Math.min(target, sim.now + PHYSICS_CHUNK));
      }
    } else if (this.speed === 'max') {
      let chunks = 0;
      do sim.runUntil(sim.now + MAX_SPEED_CHUNK);
      while (++chunks < MAX_SPEED_CHUNKS_PER_TICK && this.clock() - start < MAX_SPEED_BUDGET_MS);
    } else {
      sim.runUntil(sim.now + Math.round(realMs * 1000 * this.speed));
    }
    return this.frame(false);
  }

  /** Drops a catalogue object into the running 3D world; a rejection is posted as an error. */
  private spawn(input: unknown, actor: Actor): SimFrame | undefined {
    const physics = this.physics;
    if (physics === undefined) {
      this.post({ type: 'error', message: 'Objects can be dropped into a running 3D run only' });
      return undefined;
    }
    const parsed = parseSpawnObject(input);
    if (!parsed.ok) {
      this.post({ type: 'error', message: `${SPAWN_OBJECT} rejected: ${parsed.message}` });
      return undefined;
    }
    const objectId = physics.spawnObject(parsed.input);
    this.log.push({
      time: physics.now,
      actor,
      command: SPAWN_OBJECT,
      input: parsed.input,
      objectId,
    });
    return this.frame(!this.playing);
  }

  private load(layout: Layout, seed: number, mode: RunMode): SimFrame | undefined {
    const token = ++this.loads;
    this.sim?.dispose?.();
    this.sim = undefined;
    this.physics = undefined;
    this.log.length = 0;
    this.mode = mode;
    this.playing = false;
    this.reportedIssues = 0;
    if (mode === 'des') {
      this.sim = new FlowSimulation(buildFlowNetwork(layout, registry), { seed });
      return this.frame(true);
    }
    PhysicsFlowSimulation.create(layout, registry, { seed }).then(
      (sim) => {
        if (token !== this.loads) return sim.dispose();
        this.sim = sim;
        this.physics = sim;
        const frame = this.frame(true);
        if (frame !== undefined) this.post(frame);
      },
      (error: unknown) =>
        this.post({
          type: 'error',
          message: error instanceof Error ? error.message : String(error),
        }),
    );
    return undefined;
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
