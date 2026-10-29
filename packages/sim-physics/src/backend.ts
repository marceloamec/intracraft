// Contract every physics engine adapter must fulfil. The engine itself (Jolt or Rapier) is
// chosen in spike S1 (docs/en/decisions/ADR-0002); nothing outside this package may import it.
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
  offset: Vec3;
}

export type Shape = BoxShape | CylinderShape;

export interface BodyDesc {
  motion: 'static' | 'kinematic' | 'dynamic';
  pose: Pose;
  shapes: Shape[];
  /** Dynamic bodies only (kg). */
  mass?: number;
  /** Dynamic bodies only: centre of mass in body coordinates (m). */
  centerOfMass?: Vec3;
  friction: number;
  restitution: number;
  /** Continuous collision detection, used for pallets hitting stops. */
  ccd?: boolean;
}

export interface RayHit {
  body: BodyHandle;
  distance: number;
}

export interface PhysicsBackend {
  addBody(desc: BodyDesc): BodyHandle;
  removeBody(body: BodyHandle): void;
  setKinematicVelocity(body: BodyHandle, linear: Vec3, angular: Vec3): void;
  /** Conveyor-belt effect (plan B). Backends that cannot support it must return false. */
  setSurfaceVelocity(body: BodyHandle, velocity: Vec3): boolean;
  getPose(body: BodyHandle): Pose;
  castRay(origin: Vec3, direction: Vec3, maxDistance: number): RayHit | undefined;
  step(dt: number): void;
  /** Hash of all body states, used by the determinism test (S1-g). */
  stateHash(): string;
}
