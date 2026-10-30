// A driven deck of the 3D run: a straight conveyor (rollers or chains), or the deck of a station.
// Its control mirrors the 2D line without accumulation (ADR-0007): the deck runs at one speed for all
// its pallets, asks the next object to take its lead pallet at the decision point and otherwise
// brakes so that the pallet stops with its front at `end`. The pallets themselves move by friction.
import type { SimTime, Vec3 } from '@intracraft/core-model';
import type { PalletSink } from '@intracraft/sim-des';
import type { BodyHandle, PhysicsBackend } from '../backend';
import { rollerAngularVelocity } from '../mechanics';
import { type DeckFrame, frameAxis, scale } from './frame';
import type { GravityLine3D } from './gravity';
import { type Extent, type WorldPallet, extentOn } from './pallet-body';

const ZERO: Vec3 = { x: 0, y: 0, z: 0 };

export interface DeckSpec {
  /** Instance id: the conveyor's, or the station's for a station deck. */
  id: string;
  name: string;
  length: number;
  usableWidth: number;
  nominalSpeed: number;
  accel: number;
  decel: number;
  minGap: number;
  palletSideAlongTravel: 'long' | 'short';
}

/**
 * How the deck moves pallets: kinematic rollers spinning at ω = v / r (S01 §4), or bodies whose
 * surface moves (chain strands, plan B of ADR-0002; the rollers of a moving station platform).
 */
export type DeckDrive =
  | { kind: 'rollers'; bodies: BodyHandle[]; diameter: number }
  | { kind: 'surface'; bodies: BodyHandle[] };

export type DeckStatus = 'empty' | 'working' | 'blocked' | 'stopped';

/** The sink as the 3D run sees it: the 2D removal logic and a receiving bed per feeding deck. */
export interface SinkEnd {
  id: string;
  logic: PalletSink;
  beds: Map<Deck, BodyHandle>;
}

/** Where a deck's pallets go: another deck (conveyor or station entry), a sink or a gravity line. */
export type Exit =
  | { kind: 'deck'; deck: Deck; port: string | undefined; gap: number }
  | { kind: 'sink'; sink: SinkEnd; gap: number }
  | { kind: 'gravity'; line: GravityLine3D; gap: number };

export class Deck {
  /** +1: pallets travel toward local x = length; −1: toward x = 0 (a station leaving on its entry side). */
  direction: 1 | -1 = 1;
  /** Travel position where the lead pallet's front stops when nothing takes it (m). */
  end: number;
  exit: Exit | undefined;
  speed = 0;
  rate = 0;
  holding = false;
  status: DeckStatus = 'empty';
  /** Pallets carried (tail on this deck), in arrival order. */
  readonly pallets: WorldPallet[] = [];
  /** Pallet accepted from the previous object and still crossing onto this deck. */
  incoming: WorldPallet | undefined;
  /** Pallet handed to the exit and still crossing off this deck. */
  outgoing: WorldPallet | undefined;
  readonly stats = {
    empty: 0,
    working: 0,
    blocked: 0,
    stopped: 0,
    processed: 0,
    transitSum: 0,
    occupancy: 0,
  };

  constructor(
    readonly spec: DeckSpec,
    public frame: DeckFrame,
    readonly drive: DeckDrive,
  ) {
    this.end = spec.length;
  }

  extent(pallet: WorldPallet): Extent {
    return extentOn(pallet, this.frame, this.spec.length, this.direction);
  }

  /** The carried pallet furthest along that has not been handed over yet. */
  get lead(): WorldPallet | undefined {
    let lead: WorldPallet | undefined;
    let front = -Infinity;
    for (const pallet of this.pallets) {
      if (pallet === this.outgoing) continue;
      const f = this.extent(pallet).front;
      if (f > front) [lead, front] = [pallet, f];
    }
    return lead;
  }

  /** Travel position of the rear edge of the last pallet, or undefined when the deck is empty. */
  get rearTail(): number | undefined {
    let rear: number | undefined;
    for (const pallet of this.pallets) rear = Math.min(rear ?? Infinity, this.extent(pallet).rear);
    return rear;
  }

  /** World velocity of the conveying surface. */
  surfaceVelocity(): Vec3 {
    return scale(frameAxis(this.frame, 'x'), this.speed * this.direction);
  }

  /** Sets the roller spin or the surface velocity for the current speed. */
  applyDrive(world: PhysicsBackend): void {
    if (this.drive.kind === 'surface') {
      const velocity = this.surfaceVelocity();
      for (const body of this.drive.bodies) world.setSurfaceVelocity(body, velocity);
      return;
    }
    // Rollers turn about −z to move the top of the roller toward +x.
    const omega = rollerAngularVelocity(this.speed, this.drive.diameter) * this.direction;
    const spin = scale(frameAxis(this.frame, 'z'), -omega);
    for (const body of this.drive.bodies) world.setVelocity(body, ZERO, spin);
  }

  /** A pallet handed over: it leaves the deck's list and counts as processed. */
  release(pallet: WorldPallet, now: SimTime): void {
    const index = this.pallets.indexOf(pallet);
    if (index >= 0) this.pallets.splice(index, 1);
    if (this.outgoing === pallet) this.outgoing = undefined;
    this.stats.processed += 1;
    this.stats.transitSum += now - pallet.enteredAt;
  }

  accumulate(seconds: number): void {
    this.stats[this.status] += seconds;
    this.stats.occupancy += this.pallets.length * seconds;
  }
}
