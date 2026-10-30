// Messages between the editor (main thread) and the simulation worker (A02 §3).
import type { Layout } from '@intracraft/core-model';
import type { FlowIssue, FlowSnapshot, FlowStatistics } from '@intracraft/sim-des';

/** Simulated seconds per real second, or as fast as possible. */
export type SimSpeed = 1 | 10 | 100 | 'max';

export const SIM_SPEEDS: SimSpeed[] = [1, 10, 100, 'max'];

export type ToWorker =
  | { type: 'load'; layout: Layout; seed: number }
  | { type: 'play' }
  | { type: 'pause' }
  | { type: 'step' }
  | { type: 'setSpeed'; speed: SimSpeed };

export interface SimFrame {
  type: 'frame';
  playing: boolean;
  snapshot: FlowSnapshot;
  /** Sent a few times per second, not with every frame. */
  statistics?: FlowStatistics;
  /** Sent after loading and whenever new warnings appear. */
  issues?: FlowIssue[];
}

export type FromWorker = SimFrame | { type: 'error'; message: string };
