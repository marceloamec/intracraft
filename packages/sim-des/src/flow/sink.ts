// 2D behaviour of the pallet sink (S09 §2): removes pallets and measures cycle time.
import { type SimTime, secondsToSimTime } from '@intracraft/core-model';
import type { SinkNode } from './network';
import type { Pallet } from './pallet';

export class PalletSink {
  /** Pallets accepted and still crossing into the sink. */
  incoming = 0;
  busyUntil: SimTime = 0;
  readonly stats = { received: 0, cycleSum: 0 };

  constructor(readonly node: SinkNode) {}

  /** With a removal time the sink takes one pallet at a time; otherwise it is always free. */
  accepts(now: SimTime): boolean {
    if (this.node.removalTime === 0) return true;
    return this.incoming === 0 && now >= this.busyUntil;
  }

  receive(pallet: Pallet, now: SimTime): void {
    this.incoming -= 1;
    this.stats.received += 1;
    this.stats.cycleSum += now - pallet.createdAt;
    this.busyUntil = now + secondsToSimTime(this.node.removalTime);
  }
}
