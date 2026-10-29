// Closed-form mechanics used to configure and validate the physics mode (SI units).
import { GRAVITY } from '@intracraft/core-model';

/** Angular speed of a kinematic roller whose surface moves at `surfaceSpeed`: ω = v / r. */
export function rollerAngularVelocity(surfaceSpeed: number, diameter: number): number {
  return surfaceSpeed / (diameter / 2);
}

/** Highest acceleration friction can give a pallet on driven rollers/chains before it slips: μ·g. */
export function maxTractionAccel(mu: number): number {
  return mu * GRAVITY;
}

/**
 * Minimum friction coefficient so a pallet does not slide on a turntable:
 * μ ≥ √(a_t² + a_c²) / g, with a_t = α·r and a_c = ω²·r at the farthest pallet corner.
 */
export function turntableRequiredFriction(
  angularAccel: number,
  angularSpeed: number,
  radius: number,
): number {
  const tangential = angularAccel * radius;
  const centripetal = angularSpeed * angularSpeed * radius;
  return Math.hypot(tangential, centripetal) / GRAVITY;
}

/** Drive torque needed to spin up a turntable: τ = I·α + bearing friction torque. */
export function turntableRequiredTorque(
  totalInertia: number,
  angularAccel: number,
  frictionTorque: number,
): number {
  return totalInertia * angularAccel + frictionTorque;
}

/** Swept radius of a rectangular platform around its centre plus clearance. */
export function sweptRadius(length: number, width: number, clearance: number): number {
  return 0.5 * Math.hypot(length, width) + clearance;
}

/** Moment of inertia of a solid box about its vertical axis: m·(a² + b²) / 12. */
export function boxYawInertia(mass: number, length: number, width: number): number {
  return (mass * (length * length + width * width)) / 12;
}
