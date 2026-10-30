// 2D behaviour of the roller-to-chain transfer table (S05, T13 part 2). Two decks: rollers along x
// (A → B) and liftable chains across (C1 at −z, C2 at +z). The pallet changes direction without
// turning, so its side along travel swaps. Cycles:
//   from A: receive on rollers, centre → lift → chains carry it out through C1/C2 → lower → idle;
//   from C1/C2 (chains up): receive on chains, centre → lower → rollers carry it out through B → idle.
// The chains rest down; before a pallet can come in on the chains they lift first.
import { type SimTime, secondsToSimTime } from '@intracraft/core-model';
import type { LineConveyor } from './line-conveyor';
import type { TransferNode } from './network';
import type { Pallet } from './pallet';
import { type DeckPath, type Station, type StationStep, StationCounters, createDeck, isCentred } from './station';

const CHAIN_PORTS = ['C1', 'C2'];
/** Heading of the chains when they run towards +z (C2) or −z (C1); direction(h) = (cos h, −sin h). */
const TOWARDS_C2 = -Math.PI / 2;
const TOWARDS_C1 = Math.PI / 2;

type Phase = 'idle' | 'lifting' | 'lowering' | 'receiving' | 'discharging';
/** What happens when the running lift or lowering ends. */
type AfterStep = 'ready' | 'carryOnChains' | 'carryOnRollers' | 'cycleDone';

export class Transfer implements Station {
  readonly id: string;
  readonly rollers: LineConveyor;
  readonly chains: LineConveyor;
  readonly decks: LineConveyor[];
  phase: Phase = 'idle';
  chainsUp = false;
  transfers = 0;
  /** Direction the chains run in (TOWARDS_C1 or TOWARDS_C2). */
  private chainHeading = TOWARDS_C2;
  private afterStep: AfterStep = 'ready';
  private exitPort: string | undefined;
  private alternateIndex = 0;
  private readonly counters = new StationCounters();

  constructor(readonly node: TransferNode) {
    this.id = node.id;
    this.rollers = createDeck(node, `${node.id}#rollers`, {
      length: node.length,
      speed: node.nominalSpeed,
      accel: node.accel,
      side: node.palletSideAlongTravel,
    });
    this.chains = createDeck(node, `${node.id}#chains`, {
      length: node.usableWidth,
      speed: node.transferSpeed,
      accel: node.transferAccel,
      side: node.palletSideAlongTravel === 'long' ? 'short' : 'long',
    });
    this.decks = [this.rollers, this.chains];
  }

  entryDeck(portId: string | undefined): LineConveyor | undefined {
    if (portId === 'A' && this.node.operationMode !== 'chainsToRollers') return this.rollers;
    if (CHAIN_PORTS.includes(portId ?? '') && this.node.operationMode !== 'rollersToChains') return this.chains;
    return undefined;
  }

  get busyStatus(): 'lifting' | 'lowering' | undefined {
    return this.phase === 'lifting' || this.phase === 'lowering' ? this.phase : undefined;
  }

  canReceive(portId: string | undefined): boolean {
    const deck = this.entryDeck(portId);
    if (deck === undefined || !this.isFree()) return false;
    return deck === this.chains ? this.chainsUp : !this.chainsUp;
  }

  /** Lifts the chains for a pallet coming in on them, or lowers them for one coming in on the rollers. */
  prepareFor(portId: string | undefined): SimTime | undefined {
    const deck = this.entryDeck(portId);
    if (deck === undefined || !this.isFree() || (deck === this.chains) === this.chainsUp) return undefined;
    this.afterStep = 'ready';
    return this.startLift(deck === this.chains);
  }

  receive(portId: string | undefined, palletLength: number, now: SimTime): void {
    this.counters.startCycle(now);
    this.phase = 'receiving';
    const deck = this.entryDeck(portId) ?? this.rollers;
    if (deck === this.chains) this.chainHeading = portId === 'C1' ? TOWARDS_C2 : TOWARDS_C1;
    deck.end = (deck.node.length + palletLength) / 2;
    delete deck.node.downstream;
  }

  step(now: SimTime): StationStep {
    if (this.phase === 'receiving') {
      if (isCentred(this.rollers)) {
        this.exitPort = this.chooseChainExit();
        if (this.exitPort === undefined) return { changed: false };
        this.afterStep = 'carryOnChains';
        return { changed: true, wait: this.startLift(true) };
      }
      if (isCentred(this.chains)) {
        if (this.node.exits['B'] === undefined) return { changed: false };
        this.exitPort = 'B';
        this.afterStep = 'carryOnRollers';
        return { changed: true, wait: this.startLift(false) };
      }
    }
    if (this.phase === 'discharging' && this.rollers.pallets.length === 0 && this.chains.pallets.length === 0) {
      this.transfers += 1;
      if (this.chainsUp) {
        // The chains stay up until the pallet has left them, then go down.
        this.afterStep = 'cycleDone';
        return { changed: true, wait: this.startLift(false) };
      }
      this.finishCycle(now);
      return { changed: true };
    }
    return { changed: false };
  }

  finishTimedStep(now: SimTime): void {
    this.chainsUp = this.phase === 'lifting';
    switch (this.afterStep) {
      case 'ready':
        this.phase = 'idle';
        break;
      case 'carryOnChains':
        this.handOver(this.rollers, this.chains, this.exitPort ?? 'C2');
        break;
      case 'carryOnRollers':
        this.handOver(this.chains, this.rollers, 'B');
        break;
      case 'cycleDone':
        this.finishCycle(now);
        break;
    }
  }

  pathOf(deck: LineConveyor): DeckPath {
    if (deck === this.rollers) return { pathX: 0, pathY: 0, pathZ: 0, heading: 0 };
    const half = this.node.usableWidth / 2;
    const towardsC2 = this.chainHeading === TOWARDS_C2;
    return { pathX: this.node.length / 2, pathY: this.node.liftStroke, pathZ: towardsC2 ? -half : half, heading: this.chainHeading };
  }

  accumulate(seconds: number): void {
    this.counters.accumulate(seconds, this.phase !== 'idle');
  }

  statistics(): Record<string, number> {
    return {
      utilization: this.counters.utilization,
      transfers: this.transfers,
      meanCycleTime: this.counters.meanCycle,
      palletsProcessed: this.counters.cycles,
    };
  }

  private isFree(): boolean {
    return this.phase === 'idle' && this.rollers.pallets.length === 0 && this.chains.pallets.length === 0;
  }

  /** Starts lifting (true) or lowering (false) the chains; returns the duration (µs). */
  private startLift(up: boolean): SimTime {
    this.phase = up ? 'lifting' : 'lowering';
    return secondsToSimTime(up ? this.node.liftTime : this.node.lowerTime);
  }

  /** Moves the centred pallet to the other deck (sides along travel swap) and lets it run out. */
  private handOver(from: LineConveyor, to: LineConveyor, exitPort: string): void {
    const pallet = from.pallets.shift();
    const exit = this.node.exits[exitPort];
    if (pallet === undefined || exit === undefined) return;
    if (to === this.chains) this.chainHeading = exitPort === 'C1' ? TOWARDS_C1 : TOWARDS_C2;
    const turned: Pallet = {
      ...pallet,
      along: pallet.along === 'long' ? 'short' : 'long',
      length: pallet.width,
      width: pallet.length,
      front: (to.node.length + pallet.width) / 2,
    };
    from.holding = false;
    from.end = from.node.length;
    to.pallets.push(turned);
    to.end = to.node.length;
    to.holding = false;
    to.node.downstream = exit.target;
    to.node.downstreamGap = exit.gap;
    this.phase = 'discharging';
  }

  private finishCycle(now: SimTime): void {
    this.phase = 'idle';
    this.exitPort = undefined;
    this.counters.endCycle(now);
  }

  /** right → C2 (+z), left → C1 (−z), bidirectional → the connected chain exits in turn. */
  private chooseChainExit(): string | undefined {
    const connected = CHAIN_PORTS.filter((port) => this.node.exits[port] !== undefined);
    const wanted =
      this.node.transferDirection === 'right'
        ? ['C2']
        : this.node.transferDirection === 'left'
          ? ['C1']
          : connected;
    const candidates = wanted.filter((port) => connected.includes(port));
    if (candidates.length === 0) return undefined;
    return candidates[this.alternateIndex++ % candidates.length];
  }
}
