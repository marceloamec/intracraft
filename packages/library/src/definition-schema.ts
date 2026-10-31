// Schema of a library component definition: WHAT a component is (parameters, ports, signals).
// HOW it behaves lives in TypeScript behaviours registered by id (see registry.ts).
// Stored values use SI units (`unit`); the editor shows `displayUnit`.
import { localizedTextSchema } from '@intracraft/core-model';
import { z } from 'zod';

export const affectsSchema = z.enum(['2d', '3d', 'both', 'geometry']);

export const parameterSchema = z.object({
  key: z.string().min(1),
  label: localizedTextSchema,
  type: z.enum(['number', 'integer', 'boolean', 'enum', 'text']),
  unit: z.string().optional(),
  displayUnit: z.string().optional(),
  default: z.union([z.number(), z.boolean(), z.string()]).optional(),
  min: z.number().optional(),
  max: z.number().optional(),
  step: z.number().optional(),
  options: z.array(z.string()).optional(),
  /**
   * Options appended when the registry is built: `catalogue` (every catalogue model),
   * `catalogue:<category>,…` (models of those categories) or `materials` (S16 contact materials).
   */
  optionsFrom: z.string().optional(),
  /** Text shown for an option instead of its id (catalogue names, placement modes…). */
  optionLabels: z.record(z.string(), localizedTextSchema).optional(),
  /** Allowed values of a number parameter (shown as a dropdown), e.g. standard roller diameters. */
  choices: z.array(z.number()).optional(),
  group: z.string().min(1),
  description: localizedTextSchema,
  advanced: z.boolean().optional(),
  affects: affectsSchema,
  derived: z.boolean().optional(),
  formula: z.string().optional(),
  dependsOn: z.string().optional(),
});

/** One term of a port coordinate: a constant (m) or factor × parameter value. */
export const portTermSchema = z.union([
  z.number(),
  z.object({ param: z.string().min(1), factor: z.number().default(1) }),
]);

/** Port coordinate: a number (m), a parameter key, or a list of terms that are added up. */
export const portCoordinateSchema = z.union([
  z.number(),
  z.string().min(1),
  z.array(portTermSchema).min(1),
]);

/** Outward direction of a port in the component's local frame. */
export const facingSchema = z.enum(['+x', '-x', '+z', '-z']);

export const portSchema = z.object({
  id: z.string().min(1),
  direction: z.enum(['in', 'out', 'inout']),
  facing: facingSchema,
  local: z.object({ x: portCoordinateSchema, y: portCoordinateSchema, z: portCoordinateSchema }),
});

export const componentDefinitionSchema = z.object({
  id: z.string().regex(/^[a-z]+\.[a-z0-9_]+$/),
  version: z.string().regex(/^\d+\.\d+\.\d+$/),
  category: z.enum(['conveyors', 'sensors', 'mechanics', 'flow', 'pallets', 'control', 'scenery']),
  name: localizedTextSchema,
  description: localizedTextSchema,
  parameters: z.array(parameterSchema),
  ports: z.array(portSchema),
  signals: z.object({ inputs: z.array(z.string()), outputs: z.array(z.string()) }),
  statistics: z.array(z.string()),
  geometry: z.string(),
  behaviors: z.object({ des: z.string(), physics: z.string() }),
  validationRules: z.array(z.string()),
  presets: z.array(
    z.object({
      name: localizedTextSchema,
      values: z.record(z.string(), z.union([z.number(), z.boolean(), z.string()])),
    }),
  ),
});

export type ParameterDefinition = z.infer<typeof parameterSchema>;
export type Facing = z.infer<typeof facingSchema>;
export type PortCoordinate = z.infer<typeof portCoordinateSchema>;
export type PortDefinition = z.infer<typeof portSchema>;
export type ComponentDefinition = z.infer<typeof componentDefinitionSchema>;
