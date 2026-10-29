import { attributesSchema, type Instance, type Layout, vec3Schema } from '@intracraft/core-model';
import { type ComponentDefinition, defaultParams, validateInstance } from '@intracraft/library';
import { z } from 'zod';
import { defineCommand, fail } from './command';
import { checkParams } from './param-check';
import { normalizeYaw, paramValuesSchema } from './shared';

export const addInstance = defineCommand({
  name: 'layout.addInstance',
  input: z.object({
    definitionId: z.string().min(1),
    position: vec3Schema.default({ x: 0, y: 0, z: 0 }),
    yaw: z.number().default(0),
    params: paramValuesSchema.default({}),
    name: z.string().min(1).optional(),
    attributes: attributesSchema.default({}),
  }),
  apply(layout, input, { registry }) {
    if (!registry.has(input.definitionId)) return fail('unknownDefinition', `No definition "${input.definitionId}"`);
    const definition = registry.get(input.definitionId);
    const issues = checkParams(definition, input.params);
    if (issues.length > 0) return { ok: false, issues };

    const id = nextId(layout, definition);
    const params = { ...defaultParams(definition), ...input.params };
    const instance: Instance = {
      id,
      definitionId: definition.id,
      definitionVersion: definition.version,
      name: input.name ?? id,
      position: input.position,
      yaw: normalizeYaw(input.yaw),
      params,
      attributes: input.attributes,
    };
    return {
      ok: true,
      layout: { ...layout, instances: [...layout.instances, instance] },
      instanceId: id,
      issues: validateInstance(definition, params),
    };
  },
});

/** Readable, deterministic id: the definition's short name plus the first free number (roller1, roller2…). */
function nextId(layout: Layout, definition: ComponentDefinition): string {
  const prefix = definition.id.slice(definition.id.indexOf('.') + 1);
  const taken = new Set(layout.instances.map((i) => i.id));
  let n = 1;
  while (taken.has(`${prefix}${n}`)) n += 1;
  return `${prefix}${n}`;
}
