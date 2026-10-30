import { describe, expect, it } from 'vitest';
import { frameBracing, guideDiskPositions, guideDiskRollers, supportPositions } from './structure';

const EUR_RUNNERS = { outer: 0.1, centre: 0.145 };

describe('supportPositions', () => {
  it('puts one pair of feet near each end of a short section', () => {
    expect(supportPositions(1.6)).toEqual([expect.closeTo(0.1, 12), expect.closeTo(1.5, 12)]);
  });

  it('adds feet so that no bay is longer than 2 m', () => {
    const feet = supportPositions(6);
    expect(feet).toHaveLength(4);
    for (let i = 1; i < feet.length; i++) expect((feet[i] ?? 0) - (feet[i - 1] ?? 0)).toBeLessThanOrEqual(2);
    expect(feet.at(-1)).toBeCloseTo(5.9, 12);
  });

  it('puts a single support in the middle of a section shorter than both insets', () => {
    expect(supportPositions(0.15)).toEqual([0.075]);
  });
});

describe('frameBracing', () => {
  it('needs no bracing at the usual heights below 650 mm', () => {
    expect(frameBracing(0.52)).toEqual({ crossBeam: false, diagonal: false });
  });

  it('adds a cross beam from 650 mm and diagonal bracing from 850 mm', () => {
    expect(frameBracing(0.65)).toEqual({ crossBeam: true, diagonal: false });
    expect(frameBracing(0.85)).toEqual({ crossBeam: true, diagonal: true });
  });
});

describe('guide disks', () => {
  it('sits on every third roller, starting with the first', () => {
    expect(guideDiskRollers(14)).toEqual([0, 3, 6, 9, 12]);
  });

  it('guides the outer runners of a EUR pallet (800 mm across) from outside or inside', () => {
    expect(guideDiskPositions('outerOutside', 0.8, EUR_RUNNERS)).toEqual([expect.closeTo(-0.405, 12), expect.closeTo(0.405, 12)]);
    expect(guideDiskPositions('outerInside', 0.8, EUR_RUNNERS)).toEqual([expect.closeTo(-0.295, 12), expect.closeTo(0.295, 12)]);
  });

  it('guides the centre runner from both sides, or not at all', () => {
    expect(guideDiskPositions('centre', 0.8, EUR_RUNNERS)).toEqual([expect.closeTo(-0.0775, 12), expect.closeTo(0.0775, 12)]);
    expect(guideDiskPositions('none', 0.8, EUR_RUNNERS)).toEqual([]);
  });
});
