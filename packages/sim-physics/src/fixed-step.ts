// Fixed-timestep accumulator: physics always advances in equal steps, independent of frame rate.
// The viewer interpolates between the last two physics states with `alpha`.

export const DEFAULT_PHYSICS_DT = 1 / 120;
export const DEFAULT_MAX_STEPS_PER_FRAME = 8;

export interface StepPlan {
  steps: number;
  /** Fraction of a step left in the accumulator, in [0, 1): interpolation factor for rendering. */
  alpha: number;
}

export class FixedStepClock {
  private accumulator = 0;

  constructor(
    readonly dt: number = DEFAULT_PHYSICS_DT,
    readonly maxStepsPerFrame: number = DEFAULT_MAX_STEPS_PER_FRAME,
  ) {}

  /** Add elapsed wall time (s) and return how many fixed steps to run now. */
  advance(frameTime: number): StepPlan {
    this.accumulator += frameTime;
    let steps = Math.floor(this.accumulator / this.dt);
    if (steps > this.maxStepsPerFrame) {
      // Spiral-of-death guard: drop the backlog instead of falling further behind.
      steps = this.maxStepsPerFrame;
      this.accumulator = 0;
    } else {
      this.accumulator -= steps * this.dt;
    }
    return { steps, alpha: this.accumulator / this.dt };
  }
}
