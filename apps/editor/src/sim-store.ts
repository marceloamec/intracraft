// Simulation controls and results in the editor. The simulation itself runs in a Web Worker
// (sim/sim.worker.ts); this store sends it commands and keeps the latest frame it posted.
import type { FlowIssue, FlowSnapshot, FlowStatistics, ObjectStatus } from '@intracraft/sim-des';
import { create } from 'zustand';
import { useLayoutStore } from './layout-store';
import type { FromWorker, SimSpeed, ToWorker } from './sim/protocol';

export type RunState = 'stopped' | 'playing' | 'paused';

interface SimState {
  run: RunState;
  speed: SimSpeed;
  /** Simulation time of the latest frame (µs). */
  time: number;
  /** Number of pallets in the model in the latest frame. */
  pallets: number;
  status: Record<string, ObjectStatus>;
  /** Controller signals of the latest frame. */
  signals: Record<string, boolean>;
  statistics: FlowStatistics;
  issues: FlowIssue[];
  error: string | null;
  play(): void;
  pause(): void;
  step(): void;
  stop(): void;
  setSpeed(speed: SimSpeed): void;
}

/**
 * Latest snapshot for the 3D layer, outside React state: it changes 30 times per second and is
 * read every rendered frame (useFrame), so it must not trigger re-renders.
 */
export const liveFrame: { snapshot: FlowSnapshot | null; receivedAt: number; playing: boolean } = {
  snapshot: null,
  receivedAt: 0,
  playing: false,
};

let worker: Worker | null = null;

export const useSimStore = create<SimState>()((set, get) => {
  const send = (message: ToWorker) => connect().postMessage(message);
  const loadIfStopped = () => {
    if (get().run !== 'stopped') return;
    set({ issues: [], statistics: {}, error: null });
    const { layout, seed } = useLayoutStore.getState();
    send({ type: 'load', layout, seed });
    send({ type: 'setSpeed', speed: get().speed });
  };

  const connect = (): Worker => {
    if (worker !== null) return worker;
    worker = new Worker(new URL('./sim/sim.worker.ts', import.meta.url), { type: 'module' });
    worker.onmessage = (event: MessageEvent<FromWorker>) => {
      const message = event.data;
      if (get().run === 'stopped') return; // late frame after a stop
      if (message.type === 'error') {
        set({ error: message.message, run: 'paused' });
        send({ type: 'pause' });
        return;
      }
      liveFrame.snapshot = message.snapshot;
      liveFrame.receivedAt = performance.now();
      liveFrame.playing = message.playing;
      set({
        time: message.snapshot.time,
        pallets: message.snapshot.pallets.length,
        status: message.snapshot.status,
        signals: message.snapshot.signals,
        ...(message.statistics !== undefined && { statistics: message.statistics }),
        ...(message.issues !== undefined && { issues: message.issues }),
      });
    };
    return worker;
  };

  return {
    run: 'stopped',
    speed: 1,
    time: 0,
    pallets: 0,
    status: {},
    signals: {},
    statistics: {},
    issues: [],
    error: null,
    play() {
      loadIfStopped();
      send({ type: 'play' });
      set({ run: 'playing' });
    },
    pause() {
      if (get().run !== 'playing') return;
      send({ type: 'pause' });
      set({ run: 'paused' });
    },
    step() {
      loadIfStopped();
      send({ type: 'step' });
      set({ run: 'paused' });
    },
    stop() {
      if (get().run === 'stopped') return;
      send({ type: 'pause' });
      liveFrame.snapshot = null;
      set({ run: 'stopped', time: 0, pallets: 0, status: {}, signals: {} });
    },
    setSpeed(speed) {
      set({ speed });
      if (worker !== null) send({ type: 'setSpeed', speed });
    },
  };
});

// Results belong to the layout and seed they were computed with: any change stops the run.
useLayoutStore.subscribe((state, previous) => {
  if (state.layout !== previous.layout || state.seed !== previous.seed) useSimStore.getState().stop();
});
