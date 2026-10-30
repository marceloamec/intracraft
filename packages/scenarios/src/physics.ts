// Physics scenarios (P1–P7): the bench runs for a fixed time and one measured value is compared with the
// result expected from theory (T11, T12, T14) or with the S1 acceptance criteria (ADR-0002).
import type { ParamValues } from '@intracraft/library';
import { z } from 'zod';
import { type BenchFrame, PhysicsBench, benchDefaults } from './physics-bench';

/** Tilt that marks the start of tipping over the conveyor end (2°). */
const TIP_TILT = (2 * Math.PI) / 180;
/** Window at the end of the run over which the steady speed is averaged (s). */
const STEADY_WINDOW = 0.5;
/** Traction is fitted while the pallet speed is between these fractions of the conveyor speed. */
const SLIP_WINDOW = [0.1, 0.8] as const;

export const physicsInputsSchema = z.object({
  measure: z.enum([
    'steadySpeed',
    'tractionAccel',
    'stopFrontError',
    'reboundDistance',
    'impactEnergy',
    'restitution',
    'tipShift',
    'determinism',
  ]),
  /** Simulated time (s). */
  duration: z.number().positive(),
  /** Bench parameters that differ from the defaults. */
  params: z.record(z.string(), z.union([z.number(), z.boolean(), z.string()])),
  /** tipShift only: forward shift of the load's centre of gravity (m). */
  comShift: z.number().optional(),
});

export type PhysicsInputs = z.infer<typeof physicsInputsSchema>;

export async function runPhysicsScenario(inputs: PhysicsInputs): Promise<number> {
  const { params, duration } = inputs;
  switch (inputs.measure) {
    case 'steadySpeed':
      return withBench(params, (bench) => {
        const speeds: number[] = [];
        bench.run(
          duration,
          (f) => f.time > duration - STEADY_WINDOW && speeds.push(f.pallet.velocity.linear.x),
        );
        return speeds.reduce((sum, v) => sum + v, 0) / speeds.length;
      });
    case 'tractionAccel':
      return withBench(params, (bench) => {
        const target = Number({ ...benchDefaults(), ...params }.speed);
        const samples: [number, number][] = [];
        bench.run(duration, (f) => {
          const v = f.pallet.velocity.linear.x;
          if (v > SLIP_WINDOW[0] * target && v < SLIP_WINDOW[1] * target) samples.push([f.time, v]);
        });
        return slope(samples);
      });
    case 'stopFrontError':
      return withBench(params, (bench) => {
        const stop = bench.layout.stop;
        if (stop === undefined) throw new Error('stopFrontError needs the stop');
        return bench.run(duration).front - (stop.center.x - stop.halfExtents.x);
      });
    case 'reboundDistance':
      return withBench(params, (bench) => bench.run(duration).rebound);
    case 'impactEnergy':
      return withBench(params, (bench) => firstImpact(bench.run(duration)).energy);
    case 'restitution':
      return withBench(params, (bench) => {
        // e = rebound speed in the step after the first impact / approach speed of that impact.
        let rebound: number | undefined;
        bench.run(duration, (f) => {
          if (rebound === undefined && f.impacts.length > 0 && f.time > firstImpact(f).time) {
            rebound = -f.pallet.velocity.linear.x;
          }
        });
        const impact = firstImpact(bench.frame());
        if (rebound === undefined) throw new Error('No step after the impact');
        return rebound / impact.speed;
      });
    case 'tipShift': {
      const shift = inputs.comShift ?? 0;
      const centered = await withBench(params, (bench) => tipFront(bench, duration));
      const shifted = await withBench({ ...params, comOffsetX: shift }, (bench) =>
        tipFront(bench, duration),
      );
      return centered - shifted;
    }
    case 'determinism': {
      const first = await withBench(params, (bench) => (bench.run(duration), bench.stateHash()));
      const second = await withBench(params, (bench) => (bench.run(duration), bench.stateHash()));
      return first === second ? 1 : 0;
    }
  }
}

async function withBench<T>(params: ParamValues, measure: (bench: PhysicsBench) => T): Promise<T> {
  const bench = await PhysicsBench.create(params);
  try {
    return measure(bench);
  } finally {
    bench.dispose();
  }
}

/** Front position when the pallet starts tipping over the conveyor end. */
function tipFront(bench: PhysicsBench, duration: number): number {
  let front: number | undefined;
  bench.run(duration, (f) => {
    if (front === undefined && f.tilt > TIP_TILT) front = f.front;
  });
  if (front === undefined) throw new Error(`The pallet did not tip within ${duration} s`);
  return front;
}

function firstImpact(frame: BenchFrame) {
  const impact = frame.impacts[0];
  if (impact === undefined) throw new Error('The pallet never hit the stop');
  return impact;
}

/** Least-squares slope of y over x. */
function slope(points: [number, number][]): number {
  if (points.length < 3) throw new Error(`Too few samples to fit a slope (${points.length})`);
  const meanX = points.reduce((sum, [x]) => sum + x, 0) / points.length;
  const meanY = points.reduce((sum, [, y]) => sum + y, 0) / points.length;
  let num = 0;
  let den = 0;
  for (const [x, y] of points) {
    num += (x - meanX) * (y - meanY);
    den += (x - meanX) ** 2;
  }
  return num / den;
}
