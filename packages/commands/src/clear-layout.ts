import { z } from 'zod';
import { defineCommand } from './command';

export const clearLayout = defineCommand({
  name: 'layout.clear',
  input: z.object({}),
  apply() {
    return { ok: true, layout: { instances: [], connections: [] }, issues: [] };
  },
});
