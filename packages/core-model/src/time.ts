// Simulation time is an integer number of microseconds to avoid floating-point drift.

export type SimTime = number;

export const US_PER_S = 1_000_000;
export const US_PER_MS = 1_000;

export function secondsToSimTime(seconds: number): SimTime {
  return Math.round(seconds * US_PER_S);
}

export function simTimeToSeconds(time: SimTime): number {
  return time / US_PER_S;
}

export function msToSimTime(ms: number): SimTime {
  return Math.round(ms * US_PER_MS);
}
