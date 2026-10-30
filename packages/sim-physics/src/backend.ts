// Contract every physics engine adapter must fulfil. The engine (Jolt, ADR-0002) is imported only by
// its adapter; nothing outside this package may import it.
import type { Vec3 } from '@intracraft/core-model';

export type BodyHandle = number;

export interface Quat {
  x: number;
  y: number;
  z: number;
  w: number;
}

export interface Pose {
  position: Vec3;
  rotation: Quat;
}

export interface BoxShape {
  kind: 'box';
  halfExtents: Vec3;
  offset: Vec3;
}

export interface CylinderShape {
  kind: 'cylinder';
  radius: number;
  halfLength: number;
  /** Axis of the cylinder in body coordinates (rollers: across the conveyor). */
  axis: 'x' | 'y' | 'z';
  offset: Vec3;
}

export type Shape = BoxShape | CylinderShape;

/** Mass (kg), centre of mass (body coordinates, m) and principal moments about it (kg·m², body axes). */
export interface MassProperties {
  mass: number;
  centerOfMass: Vec3;
  inertia: Vec3;
}

export interface BodyDesc {
  motion: 'static' | 'kinematic' | 'dynamic';
  pose: Pose;
  shapes: Shape[];
  /** Contact material (S16): each contact takes friction and restitution from the pair table. */
  material: string;
  /** Required for dynamic bodies. */
  massProperties?: MassProperties;
  /**
   * Per-instance values that replace the pair table for every contact of this body. When both bodies
   * of a contact set one, the static or kinematic body (the conveyor, stop or guide) wins.
   */
  friction?: number;
  restitution?: number;
  /** Continuous collision detection, used for pallets hitting stops. */
  ccd?: boolean;
}

export interface Velocity {
  linear: Vec3;
  angular: Vec3;
}

/** A new contact between bodies that were approaching each other. */
export interface Impact {
  /** Simulated time at the end of the step in which the contact was found (s). */
  time: number;
  bodies: [BodyHandle, BodyHandle];
  /** Approach speed along the contact normal just before the contact (m/s). */
  speed: number;
  /** Kinetic energy of that approach, ½·m·v² with m the effective mass of the pair (J). */
  energy: number;
}

export interface RayHit {
  body: BodyHandle;
  distance: number;
}

/** Friction and restitution of a contact between two materials (S16 pair table). */
export type ContactPairLookup = (a: string, b: string) => { mu: number; restitution: number };

export interface PhysicsBackend {
  addBody(desc: BodyDesc): BodyHandle;
  removeBody(body: BodyHandle): void;
  /** Kinematic bodies: drive velocity (rollers, platforms). Dynamic bodies: initial condition. */
  setVelocity(body: BodyHandle, linear: Vec3, angular: Vec3): void;
  /** Conveyor-belt effect (plan B): contacts see this body's surface moving at `velocity` (world). */
  setSurfaceVelocity(body: BodyHandle, velocity: Vec3): boolean;
  getPose(body: BodyHandle): Pose;
  getVelocity(body: BodyHandle): Velocity;
  castRay(origin: Vec3, direction: Vec3, maxDistance: number): RayHit | undefined;
  step(dt: number): void;
  /** Impacts found since the previous call; the list is emptied. */
  takeImpacts(): Impact[];
  /** Hash of all body states, used by the determinism test (S1-g). */
  stateHash(): string;
  /** Frees the engine memory. The backend must not be used afterwards. */
  dispose(): void;
}
