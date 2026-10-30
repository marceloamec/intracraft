// Common shape of the stations (turntables, transfer tables): one or two decks that convey like
// short conveyors, plus timed steps (turn, lift, lower) between them. FlowSimulation drives every
// station through this interface.
import { type SimTime, US_PER_S } from '@intracraft/core-model';
import { EPS, LineConveyor } from './line-conveyor';
import type { FlowNode } from './network';

/** Where a deck's path starts and which way it runs, in the station's own frame (m, rad). */
export interface DeckPath {
  pathX: number;
  pathY: number;
  pathZ: number;
  heading: number;
}

/** What a station did when asked to move on. */
export interface StationStep {
  changed: boolean;
  /** A timed step started: call `finishTimedStep` after this many µs (0 = at once). */
  wait?: SimTime;
}

export interface Station {
  readonly id: string;
  readonly decks: LineConveyor[];
  /** Deck that takes pallets arriving through `portId`, if the port is an entry. */
  entryDeck(portId: string | undefined): LineConveyor | undefined;
  /** A pallet arriving through `portId` can be taken now. */
  canReceive(portId: string | undefined): boolean;
  /** Gets ready for `portId` (turn, lift…) if idle; returns the step time, or undefined if nothing starts. */
  prepareFor(portId: string | undefined, now: SimTime): SimTime | undefined;
  /** The pallet arriving through `portId` was accepted. */
  receive(portId: string | undefined, palletLength: number, now: SimTime): void;
  /** Starts the next step when its condition holds (pallet centred, deck emptied…). */
  step(now: SimTime): StationStep;
  /** The running timed step is over. */
  finishTimedStep(now: SimTime): void;
  /** Status shown instead of the decks' own, e.g. while turning. */
  readonly busyStatus: 'turning' | 'lifting' | 'lowering' | 'processing' | undefined;
  pathOf(deck: LineConveyor, now: SimTime): DeckPath;
  /** Adds `seconds` to the busy/idle time. */
  accumulate(seconds: number): void;
  /** Statistics keyed like the definition's `statistics` list. */
  statistics(): Record<string, number>;
}

/** Busy time and cycles of a station (a cycle runs from accepting a pallet to being free again). */
export class StationCounters {
  busy = 0;
  total = 0;
  cycles = 0;
  cycleSum = 0;
  private cycleStart: SimTime | undefined;

  accumulate(seconds: number, busy: boolean): void {
    this.total += seconds;
    if (busy) this.busy += seconds;
  }

  startCycle(now: SimTime): void {
    this.cycleStart = now;
  }

  endCycle(now: SimTime): void {
    if (this.cycleStart === undefined) return;
    this.cycles += 1;
    this.cycleSum += now - this.cycleStart;
    this.cycleStart = undefined;
  }

  get utilization(): number {
    return this.total > 0 ? this.busy / this.total : 0;
  }

  /** Mean cycle time (s). */
  get meanCycle(): number {
    return this.cycles > 0 ? this.cycleSum / this.cycles / US_PER_S : 0;
  }
}

/** A station deck: a short conveyor of `length` with one ramp for starting and stopping. */
export function createDeck(
  owner: FlowNode,
  id: string,
  deck: { length: number; speed: number; accel: number; side: 'long' | 'short' },
): LineConveyor {
  return new LineConveyor(
    {
      kind: 'line',
      id,
      name: owner.name,
      position: owner.position,
      yaw: owner.yaw,
      surfaceHeight: owner.surfaceHeight,
      downstreamGap: 0,
      length: deck.length,
      nominalSpeed: deck.speed,
      accel: deck.accel,
      decel: deck.accel,
      minGap: 0,
      palletSideAlongTravel: deck.side,
    },
    owner.id,
  );
}

/** The deck holds its only pallet stopped with its front at the stop position (centred). */
export function isCentred(deck: LineConveyor): boolean {
  const pallet = deck.front;
  return pallet !== undefined && deck.incoming === undefined && deck.holding && pallet.front >= deck.end - EPS;
}
