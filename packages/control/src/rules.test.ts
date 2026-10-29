import { describe, expect, it } from 'vitest';
import { type Rule, evaluateRules } from './rules';

// Zero-pressure accumulation (ZPA) template for two zones.
const ZPA_RULES: Rule[] = [
  {
    id: 'zone1-run',
    when: [{ signal: 'zone2.occupied', is: false }],
    then: [{ signal: 'zone1.run', value: true }],
  },
  {
    id: 'zone1-hold',
    when: [
      { signal: 'zone1.occupied', is: true },
      { signal: 'zone2.occupied', is: true },
    ],
    then: [{ signal: 'zone1.run', value: false }],
  },
];

describe('ZPA template', () => {
  it('runs zone 1 while zone 2 is free', () => {
    const writes = evaluateRules(ZPA_RULES, {}, { 'zone1.occupied': true, 'zone2.occupied': false });
    expect(writes).toEqual([{ signal: 'zone1.run', value: true, delayMs: 0, ruleId: 'zone1-run' }]);
  });

  it('holds zone 1 when both zones are occupied', () => {
    const writes = evaluateRules(ZPA_RULES, {}, { 'zone1.occupied': true, 'zone2.occupied': true });
    expect(writes.map((w) => [w.signal, w.value])).toEqual([['zone1.run', false]]);
  });
});

describe('edges', () => {
  const rules: Rule[] = [
    {
      id: 'stop-down-after-pass',
      when: [{ signal: 'pe1', edge: 'falling' }],
      then: [{ signal: 'stop1.up', value: true, delayMs: 200 }],
    },
  ];

  it('fires only on the falling edge', () => {
    expect(evaluateRules(rules, { pe1: true }, { pe1: false })).toHaveLength(1);
    expect(evaluateRules(rules, { pe1: false }, { pe1: false })).toHaveLength(0);
    expect(evaluateRules(rules, { pe1: false }, { pe1: true })).toHaveLength(0);
  });

  it('carries the configured delay', () => {
    expect(evaluateRules(rules, { pe1: true }, { pe1: false })[0]?.delayMs).toBe(200);
  });
});
