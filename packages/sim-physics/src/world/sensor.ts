// 3D behaviour of the photoelectric sensor (S06 §3D): a ray from the sensor to the reflector every
// physics step that only pallets interrupt. A beam change is stamped at the step midpoint; the output
// follows after the response time (and the filter when freed), with the 2D sensor's own logic.
import type { SimTime, Vec3 } from '@intracraft/core-model';
import type { Photoeye } from '@intracraft/sim-des';
import type { BodyHandle, PhysicsBackend } from '../backend';
import type { Deck } from './deck';
import { sub, toWorld } from './frame';

/** An output change of a sensor and when it happened. */
export interface SensorEdge {
  sensor: string;
  time: SimTime;
  detecting: boolean;
}

export class Sensor3D {
  /** Output change waiting for the response time or the filter. */
  private pending: { state: boolean; due: SimTime } | undefined;
  private readonly from: Vec3;
  private readonly span: Vec3;
  private readonly length: number;

  constructor(
    readonly logic: Photoeye,
    host: Deck,
  ) {
    const { sensorS, sensorLateral, sensorHeight, reflectorS, reflectorLateral, reflectorHeight } =
      logic.node;
    this.from = toWorld(host.frame, { x: sensorS, y: sensorHeight, z: sensorLateral });
    const to = toWorld(host.frame, { x: reflectorS, y: reflectorHeight, z: reflectorLateral });
    this.span = sub(to, this.from);
    this.length = Math.hypot(this.span.x, this.span.y, this.span.z);
  }

  get id(): string {
    return this.logic.node.id;
  }

  /** Casts the beam after a step; a change starts the delay from the step midpoint `stamp`. */
  sense(world: PhysicsBackend, isPallet: (body: BodyHandle) => boolean, stamp: SimTime): void {
    // A beam longer than the range is already reported by the network check; as in 2D it still detects.
    const blocked =
      this.length > 0 && world.castRay(this.from, this.span, this.length, isPallet) !== undefined;
    if (blocked === this.logic.raw) return;
    this.logic.raw = blocked;
    this.pending = { state: blocked, due: stamp + this.logic.delayFor(blocked) };
  }

  /** Commits a pending output change that is due at `now`; returns the edge if the output changed. */
  flush(now: SimTime): SensorEdge | undefined {
    const pending = this.pending;
    if (pending === undefined || pending.due > now) return undefined;
    this.pending = undefined;
    if (pending.state === this.logic.detecting) return undefined;
    this.logic.commit(pending.state);
    return { sensor: this.id, time: pending.due, detecting: pending.state };
  }
}
