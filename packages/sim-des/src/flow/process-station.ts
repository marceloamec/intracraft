// 2D behaviour of the process station (S11): a short deck where one pallet at a time stops and is
// processed for a time. Cycle: idle → receiving (the pallet stops centred) → processing (timed
// step) → discharging (the deck runs it out through OUT; blocked while the next object is full) → idle.
import { type Rng, type SimTime, US_PER_S, createRng, secondsToSimTime } from '@intracraft/core-model';
import type { LineConveyor } from './line-conveyor';
import type { ProcessNode } from './network';
import { type DeckPath, type Station, type StationStep, StationCounters, createDeck, isCentred } from './station';

type Phase = 'idle' | 'receiving' | 'processing' | 'discharging';

export class ProcessStation implements Station {
  readonly id: string;
  readonly deck: LineConveyor;
  readonly decks: LineConveyor[];
  phase: Phase = 'idle';
  private readonly rng: Rng;
  private readonly counters = new StationCounters();
  /** Seconds spent processing, blocked and in total. */
  private working = 0;
  private blocked = 0;
  private total = 0;
  private processed = 0;
  private processingSum: SimTime = 0;

  constructor(
    readonly node: ProcessNode,
    seed: number,
    stream: number,
  ) {
    this.id = node.id;
    this.rng = createRng(seed, stream);
    this.deck = createDeck(node, node.id, {
      length: node.length,
      speed: node.nominalSpeed,
      accel: node.accel,
      side: node.palletSideAlongTravel,
    });
    this.decks = [this.deck];
  }

  entryDeck(portId: string | undefined): LineConveyor | undefined {
    return portId === 'IN' ? this.deck : undefined;
  }

  get busyStatus(): 'processing' | undefined {
    return this.phase === 'processing' ? 'processing' : undefined;
  }

  canReceive(portId: string | undefined): boolean {
    return this.entryDeck(portId) !== undefined && this.phase === 'idle' && this.deck.pallets.length === 0;
  }

  prepareFor(): SimTime | undefined {
    return undefined;
  }

  receive(_portId: string | undefined, palletLength: number, now: SimTime): void {
    this.counters.startCycle(now);
    this.phase = 'receiving';
    this.deck.end = (this.node.length + palletLength) / 2;
    delete this.deck.node.downstream;
  }

  step(now: SimTime): StationStep {
    if (this.phase === 'receiving' && isCentred(this.deck)) {
      this.phase = 'processing';
      return { changed: true, wait: this.drawProcessingTime() };
    }
    if (this.phase === 'discharging' && this.deck.pallets.length === 0) {
      this.phase = 'idle';
      this.counters.endCycle(now);
      return { changed: true };
    }
    return { changed: false };
  }

  finishTimedStep(): void {
    this.processed += 1;
    this.openOutput();
    this.phase = 'discharging';
  }

  pathOf(): DeckPath {
    return { pathX: 0, pathY: 0, pathZ: 0, heading: 0 };
  }

  accumulate(seconds: number): void {
    this.total += seconds;
    if (this.phase === 'processing') this.working += seconds;
    else if (this.phase === 'discharging' && this.deck.status === 'blocked') this.blocked += seconds;
  }

  statistics(): Record<string, number> {
    const share = (seconds: number) => (this.total > 0 ? seconds / this.total : 0);
    return {
      utilization: share(this.working),
      blockedShare: share(this.blocked),
      idleShare: share(this.total - this.working - this.blocked),
      palletsProcessed: this.processed,
      meanProcessingTime: this.processed > 0 ? this.processingSum / this.processed / US_PER_S : 0,
      meanCycleTime: this.counters.meanCycle,
    };
  }

  /** Processing time of the next pallet (µs): fixed, normal (cut at 0) or exponential around the mean. */
  private drawProcessingTime(): SimTime {
    const { processingMode, processingTime: mean, processingStdDev } = this.node;
    let seconds = mean;
    if (processingMode === 'normal') seconds = mean + processingStdDev * this.gaussian();
    else if (processingMode === 'exponential') seconds = -mean * Math.log(1 - this.rng.next());
    const time = secondsToSimTime(Math.max(0, seconds));
    this.processingSum += time;
    return time;
  }

  /** Standard normal draw (Box–Muller). */
  private gaussian(): number {
    return Math.sqrt(-2 * Math.log(1 - this.rng.next())) * Math.cos(2 * Math.PI * this.rng.next());
  }

  /** The processed pallet may now run out through OUT. */
  private openOutput(): void {
    this.deck.end = this.deck.node.length;
    this.deck.holding = false;
    if (this.node.downstream === undefined) return;
    this.deck.node.downstream = this.node.downstream;
    this.deck.node.downstreamGap = this.node.downstreamGap;
    if (this.node.downstreamPort !== undefined) this.deck.node.downstreamPort = this.node.downstreamPort;
  }
}
