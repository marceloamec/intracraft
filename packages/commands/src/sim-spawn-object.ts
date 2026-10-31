// Simulation command `sim.spawnObject` (S17 §4, ADR-0017): drops a catalogue object into a running
// 3D world. It changes the run, not the layout: it has no undo, is not saved, and disappears with the
// run. The runner validates it here and keeps a log of it with the actor and the simulated time, so
// the same commands at the same times reproduce the same run.
import { vec3Schema } from '@intracraft/core-model';
import { z } from 'zod';
import type { Actor } from './session';

export const SPAWN_OBJECT = 'sim.spawnObject';

export const spawnObjectInputSchema = z.object({
  asset: z.string().regex(/^asset\.[a-z]+\.[a-z0-9_]+$/),
  placement: z.enum(['drop', 'rest', 'fixed']),
  /** Gap between the object's base and its support (m), S17 range. */
  dropHeight: z.number().min(0.05).max(10),
  /** Point on the support under the model origin (m). */
  position: vec3Schema,
  yaw: z.number(),
});

export type SpawnObjectInput = z.infer<typeof spawnObjectInputSchema>;

/** One applied simulation command. */
export interface SimCommandRecord {
  /** Simulated time at which it was applied (µs). */
  time: number;
  actor: Actor;
  command: typeof SPAWN_OBJECT;
  input: SpawnObjectInput;
  /** Id of the object in the run (`live1`, …). */
  objectId: string;
}

/** Validated input, or the reason it was rejected. */
export function parseSpawnObject(
  input: unknown,
): { ok: true; input: SpawnObjectInput } | { ok: false; message: string } {
  const parsed = spawnObjectInputSchema.safeParse(input);
  if (parsed.success) return { ok: true, input: parsed.data };
  return {
    ok: false,
    message: parsed.error.issues.map((i) => `${i.path.join('.')}: ${i.message}`).join('; '),
  };
}
