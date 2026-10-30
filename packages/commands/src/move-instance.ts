import { vec3Schema } from '@intracraft/core-model';
import { z } from 'zod';
import { defineCommand, findInstance, isOutcome, replaceInstance } from './command';
import { checkParams } from './param-check';
import { normalizeYaw, paramValuesSchema } from './shared';

export const moveInstance = defineCommand({
  name: 'layout.move',
  input: z.object({
    instanceId: z.string().min(1),
    position: vec3Schema,
    yaw: z.number(),
    /** Parameters that change with the move, e.g. a sensor mounted on another conveyor. */
    params: paramValuesSchema.optional(),
  }),
  apply(layout, input, { registry }) {
    const instance = findInstance(layout, input.instanceId);
    if (isOutcome(instance)) return instance;
    const params = input.params ?? {};
    const issues = checkParams(registry.get(instance.definitionId), params);
    if (issues.length > 0) return { ok: false, issues };
    const moved = {
      ...instance,
      position: input.position,
      yaw: normalizeYaw(input.yaw),
      params: { ...instance.params, ...params },
    };
    return { ok: true, layout: replaceInstance(layout, moved), issues: [] };
  },
});
