import { describe, expect, it } from 'vitest';
import { secondsToSimTime, simTimeToSeconds } from './time';
import { degToRad, mToMm, mmToM, radToDeg } from './units';

describe('unit conversions', () => {
  it('converts millimetres and metres', () => {
    expect(mmToM(1200)).toBe(1.2);
    expect(mToMm(0.076)).toBeCloseTo(76, 10);
  });

  it('converts degrees and radians', () => {
    expect(degToRad(90)).toBeCloseTo(Math.PI / 2, 12);
    expect(radToDeg(Math.PI)).toBeCloseTo(180, 12);
  });
});

describe('simulation time', () => {
  it('stores 12.25 s as an exact integer of microseconds', () => {
    expect(secondsToSimTime(12.25)).toBe(12_250_000);
    expect(simTimeToSeconds(12_250_000)).toBe(12.25);
  });
});
