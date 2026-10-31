// Messages between the editor (main thread) and the simulation worker (A02 §3).
import type { Actor } from '@intracraft/commands';
import type { Layout } from '@intracraft/core-model';
import type { FlowIssue, FlowSnapshot, FlowStatistics } from '@intracraft/sim-des';
import type { PalletLook, Pose, SceneryView } from '@intracraft/sim-physics';

/** Simulated seconds per real second, or as fast as possible. */
export type SimSpeed = 1 | 10 | 100 | 'max';

export const SIM_SPEEDS: SimSpeed[] = [1, 10, 100, 'max'];

/** 2D discrete-event run, or 3D run with physics (ADR-0016). */
export type RunMode = 'des' | 'physics';

export type ToWorker =
  | { type: 'load'; layout: Layout; seed: number; mode?: RunMode }
  | { type: 'play' }
  | { type: 'pause' }
  | { type: 'step' }
  | { type: 'setSpeed'; speed: SimSpeed }
  /** Simulation command `sim.spawnObject` (S17 §4): validated by the runner, 3D runs only. */
  | { type: 'spawn'; input: unknown; actor?: Actor };

/**
 * The 2D snapshot; a 3D run adds the real pose of every pallet (underside centre and rotation), the
 * catalogue models to draw for each pallet and the scenery objects that can move (S09 §1.1, S17).
 */
export type SimSnapshot = FlowSnapshot & {
  poses?: Record<number, Pose>;
  looks?: Record<number, PalletLook>;
  objects?: Record<string, SceneryView>;
};

export interface SimFrame {
  type: 'frame';
  playing: boolean;
  snapshot: SimSnapshot;
  /** Sent a few times per second, not with every frame. */
  statistics?: FlowStatistics;
  /** Sent after loading and whenever new warnings appear. */
  issues?: FlowIssue[];
}

export type FromWorker = SimFrame | { type: 'error'; message: string };
