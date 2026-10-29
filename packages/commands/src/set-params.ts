import { validateInstance } from '@intracraft/library';
import { z } from 'zod';
import { defineCommand, findInstance, isOutcome, replaceInstance } from './command';
import { checkParams } from './param-check';
import { paramValuesSchema } from './shared';

export const setParams = defineCommand({
  name: 'layout.setParams',
  input: z.object({ instanceId: z.string().min(1), params: paramValuesSchema }),
  apply(layout, input, { registry }) {
    const instance = findInstance(layout, input.instanceId);
    if (isOutcome(instance)) return instance;
    const definition = registry.get(instance.definitionId);
    const issues = checkParams(definition, input.params);
    if (issues.length > 0) return { ok: false, issues };
    const params = { ...instance.params, ...input.params };
    return {
      ok: true,
      layout: replaceInstance(layout, { ...instance, params }),
      issues: validateInstance(definition, params),
    };
  },
});
