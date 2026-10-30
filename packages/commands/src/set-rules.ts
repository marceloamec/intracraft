// Replaces the controller rules (A08). Rule ids must be unique; signals of objects that are not in
// the layout are accepted with a warning, so rules can be written before the objects exist.
import { ruleSchema } from '@intracraft/core-model';
import type { ValidationIssue } from '@intracraft/library';
import { z } from 'zod';
import { defineCommand, fail } from './command';

export const setRules = defineCommand({
  name: 'control.setRules',
  input: z.object({ rules: z.array(ruleSchema) }),
  apply(layout, { rules }) {
    const ids = rules.map((r) => r.id);
    const duplicate = ids.find((id, i) => ids.indexOf(id) !== i);
    if (duplicate !== undefined) return fail('duplicateRuleId', `Two rules have the id "${duplicate}"`);

    const known = new Set(layout.instances.map((i) => i.id));
    const issues: ValidationIssue[] = [];
    for (const rule of rules) {
      const signals = [...rule.when.map((c) => c.signal), ...rule.then.map((a) => a.signal)];
      for (const signal of signals) {
        const owner = signal.slice(0, signal.indexOf('.'));
        if (!known.has(owner)) {
          issues.push({ rule: 'unknownSignal', severity: 'warning', message: `${rule.id}: "${signal}" does not belong to an object of the layout` });
        }
      }
    }
    return { ok: true, layout: { ...layout, rules }, issues };
  },
});
