import { z } from 'zod';
import { defineCommand, findInstance, isOutcome, replaceInstance } from './command';

export const setAttributes = defineCommand({
  name: 'layout.setAttributes',
  input: z.object({
    instanceId: z.string().min(1),
    name: z.string().min(1).optional(),
    /** Replaces all free attributes of the instance. */
    attributes: z.record(z.string().min(1), z.union([z.number(), z.boolean(), z.string()])).optional(),
  }),
  apply(layout, input) {
    const instance = findInstance(layout, input.instanceId);
    if (isOutcome(instance)) return instance;
    const updated = {
      ...instance,
      name: input.name ?? instance.name,
      attributes: input.attributes ?? instance.attributes,
    };
    return { ok: true, layout: replaceInstance(layout, updated), issues: [] };
  },
});
