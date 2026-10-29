import { describe, expect, it } from 'vitest';
import { minRollersUnder, rollerLayout } from './rollers';

describe('rollerLayout', () => {
  it('places 22 rollers (Ø76, pitch 133 mm) on a 3 m conveyor, centred', () => {
    const layout = rollerLayout(3, 0.133, 0.076);
    expect(layout.count).toBe(22);
    const first = layout.positions[0] ?? NaN;
    const last = layout.positions[layout.count - 1] ?? NaN;
    expect(first).toBeCloseTo(3 - last, 12);
  });

  it('leaves a 57 mm gap between rollers (transfer-table chain check)', () => {
    expect(rollerLayout(3, 0.133, 0.076).gap).toBeCloseTo(0.057, 12);
  });

  it('rejects rollers larger than the pitch', () => {
    expect(() => rollerLayout(3, 0.07, 0.076)).toThrow();
  });
});

describe('minRollersUnder', () => {
  it('keeps at least 3 rollers under a 1200 mm pallet side at 133 mm pitch', () => {
    expect(minRollersUnder(1.2, 0.133)).toBeGreaterThanOrEqual(3);
  });
});
