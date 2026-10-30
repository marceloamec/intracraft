// 2D behaviour of the pallet source (S09 §1). A pallet that finds the entry occupied either waits
// (the next interval then counts from the moment it is placed) or is rejected.
import { type Rng, type SimTime, createRng, secondsToSimTime } from '@intracraft/core-model';
import type { SourceNode } from './network';

export class PalletSource {
  /** Time the waiting pallet became due, or undefined when none is waiting. */
  dueSince: SimTime | undefined;
  readonly stats = { created: 0, rejected: 0, waiting: 0 };
  private readonly rng: Rng;

  constructor(
    readonly node: SourceNode,
    seed: number,
    stream: number,
  ) {
    this.rng = createRng(seed, stream);
  }

  /** True while more pallets may be created. */
  get active(): boolean {
    return this.node.maxPallets === 0 || this.stats.created + this.stats.rejected < this.node.maxPallets;
  }

  /** Delay before the next pallet: the interval, or an exponential draw with the interval as mean. */
  nextDelay(): SimTime {
    if (this.node.generationMode === 'fixedInterval') return secondsToSimTime(this.node.interval);
    return secondsToSimTime(-this.node.interval * Math.log(1 - this.rng.next()));
  }

  placed(now: SimTime): void {
    this.stats.created += 1;
    this.stats.waiting += now - (this.dueSince ?? now);
    this.dueSince = undefined;
  }

  rejected(): void {
    this.stats.rejected += 1;
    this.dueSince = undefined;
  }
}
