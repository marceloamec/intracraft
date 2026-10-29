import { degToRad } from '@intracraft/core-model';
import { describe, expect, it } from 'vitest';
import { FixedStepClock } from './fixed-step';
import {
  boxYawInertia,
  maxTractionAccel,
  rollerAngularVelocity,
  sweptRadius,
  turntableRequiredFriction,
  turntableRequiredTorque,
} from './mechanics';

describe('closed-form mechanics', () => {
  it('drives a Ø76 mm roller at ≈ 6.58 rad/s for 0.25 m/s', () => {
    expect(rollerAngularVelocity(0.25, 0.076)).toBeCloseTo(6.5789, 3);
  });

  it('limits traction acceleration to μ·g (μ = 0.35 → 3.43 m/s²)', () => {
    expect(maxTractionAccel(0.35)).toBeCloseTo(3.4335, 3);
  });

  it('needs μ ≈ 0.04 on a turntable with the default ramps (EUR corner r = 0.72 m)', () => {
    const mu = turntableRequiredFriction(degToRad(30), degToRad(15), 0.72);
    expect(mu).toBeGreaterThan(0.035);
    expect(mu).toBeLessThan(0.045);
  });

  it('adds inertia and bearing friction for the drive torque', () => {
    expect(turntableRequiredTorque(800, 0.5, 20)).toBe(420);
  });

  it('computes the swept radius of a 1400 × 1200 mm platform with 50 mm clearance', () => {
    expect(sweptRadius(1.4, 1.2, 0.05)).toBeCloseTo(0.972, 3);
  });

  it('computes the yaw inertia of a box', () => {
    expect(boxYawInertia(12, 1, 1)).toBe(2);
  });
});

describe('FixedStepClock', () => {
  it('runs whole steps and keeps the remainder for interpolation', () => {
    const clock = new FixedStepClock(0.01);
    const plan = clock.advance(0.025);
    expect(plan.steps).toBe(2);
    expect(plan.alpha).toBeCloseTo(0.5, 9);
  });

  it('caps the number of steps per frame after a long stall', () => {
    const clock = new FixedStepClock(0.01, 4);
    expect(clock.advance(1).steps).toBe(4);
    expect(clock.advance(0.005).steps).toBe(0);
  });
});
