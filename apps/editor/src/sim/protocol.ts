// Messages between the editor (main thread) and the simulation worker (A02 §3).
import type { Layout } from '@intracraft/core-model';
import type { FlowIssue, FlowSnapshot, FlowStatistics } from '@intracraft/sim-des';
import type { Pose } from '@intracraft/sim-physics';

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
  | { type: 'setSpeed'; speed: SimSpeed };

/** The 2D snapshot; a 3D run adds the real pose of every pallet (underside centre and rotation). */
export type SimSnapshot = FlowSnapshot & { poses?: Record<number, Pose> };

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
