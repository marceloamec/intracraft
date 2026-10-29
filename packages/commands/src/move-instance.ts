import { vec3Schema } from '@intracraft/core-model';
import { z } from 'zod';
import { defineCommand, findInstance, isOutcome, replaceInstance } from './command';
import { normalizeYaw } from './shared';

export const moveInstance = defineCommand({
  name: 'layout.move',
  input: z.object({ instanceId: z.string().min(1), position: vec3Schema, yaw: z.number() }),
  apply(layout, input) {
    const instance = findInstance(layout, input.instanceId);
    if (isOutcome(instance)) return instance;
    const moved = { ...instance, position: input.position, yaw: normalizeYaw(input.yaw) };
    return { ok: true, layout: replaceInstance(layout, moved), issues: [] };
  },
});
