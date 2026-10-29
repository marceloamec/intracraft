import { z } from 'zod';
import { defineCommand, findInstance, isOutcome } from './command';

export const removeInstance = defineCommand({
  name: 'layout.remove',
  input: z.object({ instanceId: z.string().min(1) }),
  apply(layout, input) {
    const instance = findInstance(layout, input.instanceId);
    if (isOutcome(instance)) return instance;
    // Connections are recomputed by the session, so the removed instance's links disappear too.
    return { ok: true, layout: { ...layout, instances: layout.instances.filter((i) => i !== instance) }, issues: [] };
  },
});
