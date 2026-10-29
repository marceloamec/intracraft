import { describe, expect, it } from 'vitest';
import { SCENARIOS, findScenario, runScenario } from './analytic';

describe('analytic reference scenarios', () => {
  it('contains T1 to T5', () => {
    expect(SCENARIOS.map((s) => s.id)).toEqual(['T1', 'T2', 'T3', 'T4', 'T5']);
  });

  it.each(SCENARIOS.map((s) => [s.id, s] as const))('%s passes within tolerance', (_id, scenario) => {
    const result = runScenario(scenario);
    expect(result.passed, `${result.id}: ${result.actual} vs ${result.expected}`).toBe(true);
  });

  it('finds scenarios case-insensitively', () => {
    expect(findScenario('t3')?.id).toBe('T3');
  });
});
