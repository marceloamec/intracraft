// Project file schema (.intracraft.json). Stored values use SI units; the UI converts to mm and degrees.
import { z } from 'zod';

export const SCHEMA_VERSION = 1;

export const vec3Schema = z.object({ x: z.number(), y: z.number(), z: z.number() });

// English is required; Spanish and German are optional and fall back to English (ADR-0003 amendment).
export const localizedTextSchema = z.object({
  en: z.string().min(1),
  es: z.string().optional(),
  de: z.string().optional(),
});

/** Free key/value attributes the user attaches to an instance (e.g. zone = "A", line = 3). */
export const attributesSchema = z.record(z.string(), z.union([z.number(), z.boolean(), z.string()]));

export const instanceSchema = z.object({
  id: z.string().min(1),
  definitionId: z.string().min(1),
  definitionVersion: z.string().min(1),
  name: z.string(),
  position: vec3Schema,
  yaw: z.number(),
  params: z.record(z.string(), z.union([z.number(), z.boolean(), z.string()])),
  attributes: attributesSchema.default({}),
});

export const portRefSchema = z.object({
  instanceId: z.string().min(1),
  portId: z.string().min(1),
});

export const connectionSchema = z.object({
  from: portRefSchema,
  to: portRefSchema,
  gap: z.number().nonnegative().default(0.02),
});

export const layoutSchema = z.object({
  instances: z.array(instanceSchema),
  connections: z.array(connectionSchema),
});

export const projectSchema = z.object({
  schemaVersion: z.literal(SCHEMA_VERSION),
  name: z.string(),
  seed: z.number().int().nonnegative(),
  layout: layoutSchema,
});

export type Vec3 = z.infer<typeof vec3Schema>;
export type LocalizedText = z.infer<typeof localizedTextSchema>;
export type Attributes = z.infer<typeof attributesSchema>;
export type Instance = z.infer<typeof instanceSchema>;
export type Connection = z.infer<typeof connectionSchema>;
export type Layout = z.infer<typeof layoutSchema>;
export type Project = z.infer<typeof projectSchema>;

export type Language = 'en' | 'es' | 'de';

/** Text in the requested language, falling back to English when no translation exists. */
export function localize(text: LocalizedText, language: Language): string {
  return text[language] ?? text.en;
}
