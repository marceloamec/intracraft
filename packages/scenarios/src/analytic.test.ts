import { describe, expect, it } from 'vitest';
import { SCENARIOS, findScenario, runScenario, scenarioSchema } from './analytic';

describe('reference scenarios', () => {
  it('contains T1 to T5, the 2D layout runs, the five-conveyor reference, the process station and P1 to P11', () => {
    expect(SCENARIOS.map((s) => s.id)).toEqual([
      'T1',
      'T2',
      'T3',
      'T4',
      'T5',
      'T1-2D',
      'T2-2D',
      'REF-5',
      'PROC-1',
      'P1',
      'P2-steel',
      'P2-rubber',
      'P2-plastic',
      'P3',
      'P3-rebound',
      'P4',
      'P5-steel',
      'P5-pu',
      'P5-rubber',
      'P6',
      'P7',
      'P8',
      'P8-jitter',
      'P8-planA',
      'P9',
      'P9-yaw',
      'P9-steel',
      'P10',
      'P10-fast',
      'P11',
    ]);
  });

  it.each(SCENARIOS.map((s) => [s.id, s] as const))(
    '%s passes within tolerance',
    async (_id, scenario) => {
      const result = await runScenario(scenario);
      expect(result.passed, `${result.id}: ${result.actual} vs ${result.expected}`).toBe(true);
    },
  );

  it('finds scenarios case-insensitively', () => {
    expect(findScenario('t3')?.id).toBe('T3');
  });

  it('requires exactly one tolerance', () => {
    const base = {
      id: 'X',
      title: { en: 'x' },
      unit: 'J',
      expected: 1,
      kind: 'impactEnergy',
      inputs: { mass: 2, speed: 1 },
    };
    expect(() => scenarioSchema.parse(base)).toThrow(/tolerance/i);
    expect(() =>
      scenarioSchema.parse({ ...base, relativeTolerance: 0.1, absoluteTolerance: 0.1 }),
    ).toThrow(/tolerance/i);
    expect(() => scenarioSchema.parse({ ...base, absoluteTolerance: 0 })).not.toThrow();
  });
});
