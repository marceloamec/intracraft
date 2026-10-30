// Replaces the whole layout, e.g. with one read from a project file. Atomic like every command:
// one unknown definition, duplicate id or invalid parameter rejects the whole load.
import { type Instance, layoutSchema } from '@intracraft/core-model';
import { type ValidationIssue, defaultParams, validateInstance } from '@intracraft/library';
import { z } from 'zod';
import { defineCommand } from './command';
import { checkParams } from './param-check';

export const loadLayout = defineCommand({
  name: 'layout.load',
  input: z.object({ layout: layoutSchema }),
  apply(_layout, { layout }, { registry }) {
    const errors: ValidationIssue[] = [];
    const warnings: ValidationIssue[] = [];
    const seen = new Set<string>();
    const instances: Instance[] = [];

    for (const instance of layout.instances) {
      if (seen.has(instance.id)) {
        errors.push(issue('duplicateInstanceId', `Two objects have the id "${instance.id}"`));
        continue;
      }
      seen.add(instance.id);
      if (!registry.has(instance.definitionId)) {
        errors.push(issue('unknownDefinition', `${instance.id}: no definition "${instance.definitionId}"`));
        continue;
      }
      const definition = registry.get(instance.definitionId);
      const problems = checkParams(definition, instance.params).map((p) => ({ ...p, message: `${instance.id}: ${p.message}` }));
      if (problems.length > 0) {
        errors.push(...problems);
        continue;
      }
      // Parameters added to the definition after the file was saved take their defaults.
      const params = { ...defaultParams(definition), ...instance.params };
      instances.push({ ...instance, params });
      warnings.push(...validateInstance(definition, params).map((w) => ({ ...w, message: `${instance.id}: ${w.message}` })));
    }

    if (errors.length > 0) return { ok: false, issues: errors };
    // Connections are always derived again from the geometry by the session.
    return { ok: true, layout: { instances, connections: [], ...(layout.rules && { rules: layout.rules }) }, issues: warnings };
  },
});

function issue(rule: string, message: string): ValidationIssue {
  return { rule, severity: 'error', message };
}
