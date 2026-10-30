import { describe, expect, it } from 'vitest';
import { SCENARIOS, findScenario, runScenario } from './analytic';

describe('analytic reference scenarios', () => {
  it('contains T1 to T5, the 2D layout runs of T1 and T2, the five-conveyor reference and the process station', () => {
    expect(SCENARIOS.map((s) => s.id)).toEqual(['T1', 'T2', 'T3', 'T4', 'T5', 'T1-2D', 'T2-2D', 'REF-5', 'PROC-1']);
  });

  it.each(SCENARIOS.map((s) => [s.id, s] as const))('%s passes within tolerance', (_id, scenario) => {
    const result = runScenario(scenario);
    expect(result.passed, `${result.id}: ${result.actual} vs ${result.expected}`).toBe(true);
  });

  it('finds scenarios case-insensitively', () => {
    expect(findScenario('t3')?.id).toBe('T3');
  });
});
