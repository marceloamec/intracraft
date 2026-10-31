// 2D behaviour of the manual lift table, the scissor lift table and the pallet elevator (S12–S14,
// ADR-0013): one deck takes a pallet at the entry height, the station moves it to the exit height,
// the deck runs it out and the station goes back empty. Cycle:
//   idle → receiving (the pallet stops centred) → moving (timed step) → discharging (blocked while
//   the next object is full) → returning (timed step) → idle.
// The deck conveys both ways: a pallet that leaves on the side it came in (C) runs out backwards
// without turning; its former tail leads.
import { type SimTime, US_PER_S, secondsToSimTime } from '@intracraft/core-model';
import { type MoveProfile, profilePositionAt, profileTime } from '../kinematics';
import { EPS, type LineConveyor } from './line-conveyor';
import type { LiftNode, LiftPort } from './network';
import { type DeckPath, type Station, type StationStep, StationCounters, createDeck, isCentred } from './station';

type LiftPhase = 'idle' | 'receiving' | 'moving' | 'discharging' | 'returning';

interface VerticalMove {
  start: SimTime;
  from: number;
  to: number;
  profile: MoveProfile;
}

/** The exit a lift uses: the first connected one in the definition's order of preference. */
interface LiftExit {
  port: LiftPort;
  target: string;
  gap: number;
  targetPort: string;
}

export class LiftStation implements Station {
  readonly id: string;
  readonly deck: LineConveyor;
  readonly decks: LineConveyor[];
  phase: LiftPhase = 'idle';
  private height: number;
  private move: VerticalMove | undefined;
  private readonly exit: LiftExit | undefined;
  private readonly counters = new StationCounters();
  /** Seconds in a cycle (not blocked), blocked and in total. */
  private busy = 0;
  private blocked = 0;
  private total = 0;
  private moved = 0;

  constructor(readonly node: LiftNode) {
    this.id = node.id;
    this.deck = createDeck(node, node.id, {
      length: node.length,
      speed: node.nominalSpeed,
      accel: node.accel,
      side: node.palletSideAlongTravel,
    });
    this.decks = [this.deck];
    this.height = node.entry.height;
    this.exit = chooseExit(node);
  }

  entryDeck(portId: string | undefined): LineConveyor | undefined {
    return portId === this.node.entry.portId ? this.deck : undefined;
  }

  get busyStatus(): 'lifting' | 'lowering' | undefined {
    if (this.move === undefined) return undefined;
    return this.move.to > this.move.from ? 'lifting' : 'lowering';
  }

  canReceive(portId: string | undefined): boolean {
    return this.entryDeck(portId) !== undefined && this.phase === 'idle' && this.deck.pallets.length === 0;
  }

  prepareFor(): SimTime | undefined {
    return undefined;
  }

  /** The pallet was accepted: it stops with its centre on the deck centre. */
  receive(_portId: string | undefined, palletLength: number, now: SimTime): void {
    this.counters.startCycle(now);
    this.phase = 'receiving';
    this.deck.end = (this.node.length + palletLength) / 2;
    delete this.deck.node.downstream;
  }

  /** Centred pallet: move to the exit height. Emptied deck: go back to the entry height. */
  step(now: SimTime): StationStep {
    if (this.phase === 'receiving' && isCentred(this.deck)) {
      this.phase = 'moving';
      return { changed: true, wait: this.startMove(this.exit?.port.height ?? this.height, now) };
    }
    if (this.phase === 'discharging' && this.exit !== undefined && this.deck.pallets.length === 0) {
      this.moved += 1;
      this.phase = 'returning';
      return { changed: true, wait: this.startMove(this.node.entry.height, now) };
    }
    return { changed: false };
  }

  finishTimedStep(now: SimTime): void {
    if (this.move !== undefined) this.height = this.move.to;
    this.move = undefined;
    if (this.phase === 'moving') {
      this.phase = 'discharging';
      this.openOutput();
    } else if (this.phase === 'returning') {
      this.phase = 'idle';
      this.counters.endCycle(now);
    }
  }

  /** Deck height at time `t` (µs), following the vertical profile during a move (m). */
  heightAt(t: SimTime): number {
    const move = this.move;
    if (move === undefined) return this.height;
    const covered = profilePositionAt(move.profile, (t - move.start) / US_PER_S);
    return move.from + Math.sign(move.to - move.from) * covered;
  }

  /** The deck runs from the entry end; while discharging it runs towards the exit end. */
  pathOf(_deck: LineConveyor, now: SimTime): DeckPath {
    const fromEnd = this.phase === 'discharging' && this.exit !== undefined ? !this.exit.port.atEnd : this.node.entry.atEnd;
    return {
      pathX: fromEnd ? this.node.length : 0,
      pathY: this.heightAt(now),
      pathZ: 0,
      heading: fromEnd ? Math.PI : 0,
    };
  }

  accumulate(seconds: number): void {
    this.total += seconds;
    if (this.phase === 'idle') return;
    if (this.phase === 'discharging' && this.deck.status === 'blocked') this.blocked += seconds;
    else this.busy += seconds;
  }

  statistics(): Record<string, number> {
    const share = (seconds: number) => (this.total > 0 ? seconds / this.total : 0);
    return {
      utilization: share(this.busy),
      blockedShare: share(this.blocked),
      idleShare: share(this.total - this.busy - this.blocked),
      palletsMoved: this.moved,
      meanCycleTime: this.counters.meanCycle,
    };
  }

  /** Starts a vertical move to `to` (m); returns its time (µs), 0 when already there. */
  private startMove(to: number, now: SimTime): SimTime {
    const distance = Math.abs(to - this.height);
    if (distance < EPS) return 0;
    const up = to > this.height;
    const profile = {
      distance,
      maxSpeed: up ? this.node.upSpeed : this.node.downSpeed,
      accel: this.node.verticalAccel,
      decel: this.node.verticalAccel,
    };
    this.move = { start: now, from: this.height, to, profile };
    return secondsToSimTime(profileTime(profile));
  }

  /**
   * The moved pallet may run out through the exit. Seen from the exit side the centred pallet is
   * still centred, so only the direction changes for a C exit. Without an exit it stays on the deck.
   */
  private openOutput(): void {
    const exit = this.exit;
    if (exit === undefined) return;
    this.deck.end = this.node.length;
    this.deck.holding = false;
    this.deck.node.downstream = exit.target;
    this.deck.node.downstreamGap = exit.gap;
    this.deck.node.downstreamPort = exit.targetPort;
  }
}

function chooseExit(node: LiftNode): LiftExit | undefined {
  for (const port of node.exitPorts) {
    const connected = node.exits[port.portId];
    if (connected !== undefined) return { port, target: connected.target, gap: connected.gap, targetPort: connected.port };
  }
  return undefined;
}
