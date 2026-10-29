import { degToRad } from '@intracraft/core-model';
import { describe, expect, it } from 'vitest';
import {
  impactEnergy,
  maxThroughput,
  palletCapacity,
  positionAt,
  profileTime,
  timeToReach,
  transferCycleTime,
} from './kinematics';

// Reference tests from the MVP plan, section 15.2 (tolerance < 1 %).
const REL_TOL = 0.01;

describe('analytic reference tests', () => {
  it('T1: 3 m from rest at 0.25 m/s, a = 0.5 m/s², no braking → 12.25 s', () => {
    const t = profileTime({ distance: 3, maxSpeed: 0.25, accel: 0.5 });
    expectWithin(t, 12.25);
  });

  it('T2: EUR long side, gap 0.15 m, 0.25 m/s → ≈ 0.185 pallets/s (≈ 667/h)', () => {
    const rate = maxThroughput(0.25, 1.2, 0.15);
    expectWithin(rate, 0.25 / 1.35);
    expectWithin(rate * 3600, 666.7);
  });

  it('T3: 90° turn at 15 °/s with 30 °/s² → 6.5 s', () => {
    const t = profileTime({
      distance: degToRad(90),
      maxSpeed: degToRad(15),
      accel: degToRad(30),
      decel: degToRad(30),
    });
    expectWithin(t, 6.5);
  });

  it('T4: transfer cycle 1 m, 0.2 m/s, 0.5 m/s², 1 s lift and 1 s lower → 7.4 s', () => {
    const t = transferCycleTime({ liftTime: 1, lowerTime: 1, distance: 1, speed: 0.2, accel: 0.5 });
    expectWithin(t, 7.4);
  });

  it('T5: impact of 1025 kg at 0.25 m/s → 32 J', () => {
    expectWithin(impactEnergy(1025, 0.25), 32.03);
  });
});

describe('profileTime', () => {
  it('uses a triangular profile when cruise speed is never reached', () => {
    // ω²/α = 1 rad ≥ θ = 0.5 rad → t = 2·√(θ/α)
    const t = profileTime({ distance: 0.5, maxSpeed: 1, accel: 1, decel: 1 });
    expect(t).toBeCloseTo(2 * Math.sqrt(0.5), 12);
  });

  it('returns 0 for a zero distance', () => {
    expect(profileTime({ distance: 0, maxSpeed: 1, accel: 1 })).toBe(0);
  });

  it('rejects non-positive speed', () => {
    expect(() => profileTime({ distance: 1, maxSpeed: 0, accel: 1 })).toThrow();
  });
});

describe('kinematic segments', () => {
  it('predicts when an accelerating front reaches a position', () => {
    const seg = { t0: 0, s0: 0, v0: 0, a: 0.5 };
    const t = timeToReach(seg, 0.0625); // end of the 0.25 m/s ramp
    expect(t).toBeCloseTo(0.5, 12);
    expect(positionAt(seg, 0.5)).toBeCloseTo(0.0625, 12);
  });

  it('returns undefined when a stopped pallet never reaches the target', () => {
    expect(timeToReach({ t0: 0, s0: 0, v0: 0, a: 0 }, 1)).toBeUndefined();
  });

  it('returns undefined when a decelerating pallet stops short', () => {
    // stops after v0²/(2|a|) = 0.0625 m
    expect(timeToReach({ t0: 0, s0: 0, v0: 0.25, a: -0.5 }, 0.1)).toBeUndefined();
  });
});

describe('capacity', () => {
  it('fits 2 EUR pallets (1.2 m + 0.15 m) on a 3 m conveyor', () => {
    expect(palletCapacity(3, 1.2, 0.15)).toBe(2);
  });
});

function expectWithin(actual: number, expected: number): void {
  expect(Math.abs(actual - expected) / expected).toBeLessThan(REL_TOL);
}
