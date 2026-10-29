import { describe, expect, it } from 'vitest';
import { chainLayout, transferChainPositions } from './chains';
import { rollerLayout } from './rollers';

describe('chainLayout', () => {
  it('places 3 strands symmetric about the axis, the outer ones 100 mm inside the edges', () => {
    const positions = chainLayout(1.3, 3);
    expect(positions).toHaveLength(3);
    expect(positions[0]).toBeCloseTo(-0.55, 12);
    expect(positions[1]).toBeCloseTo(0, 12);
    expect(positions[2]).toBeCloseTo(0.55, 12);
  });

  it('rejects fewer than 2 strands or a width smaller than the insets', () => {
    expect(() => chainLayout(1.3, 1)).toThrow();
    expect(() => chainLayout(0.15, 3)).toThrow();
  });
});

describe('transferChainPositions', () => {
  it('puts every transfer chain inside a gap between rollers (Ø89, 206.3 mm, 1.4 m)', () => {
    const rollers = rollerLayout(1.4, 0.2063, 0.089);
    const chains = transferChainPositions(1.4, 0.2063, 0.089, 3);
    expect(chains).toHaveLength(3);
    for (const x of chains) {
      const clearance = Math.min(...rollers.positions.map((r) => Math.abs(r - x)));
      expect(clearance).toBeGreaterThanOrEqual(0.089 / 2 + 0.04 / 2);
    }
  });

  it('spreads the chains from the first to the last gap', () => {
    const rollers = rollerLayout(1.4, 0.2063, 0.089).positions;
    const chains = transferChainPositions(1.4, 0.2063, 0.089, 3);
    expect(chains[0]).toBeCloseTo(((rollers[0] ?? NaN) + (rollers[1] ?? NaN)) / 2, 12);
    expect(chains[2]).toBeCloseTo(((rollers.at(-2) ?? NaN) + (rollers.at(-1) ?? NaN)) / 2, 12);
  });

  it('rejects more chains than gaps', () => {
    expect(() => transferChainPositions(1.0, 0.2063, 0.089, 5)).toThrow();
  });
});
