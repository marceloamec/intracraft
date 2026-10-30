// Parameter values of an instance: library defaults, the instance's own values, and derived values.
import type { ComponentDefinition } from './definition-schema';
import { defaultParams } from './registry';

export type ParamValues = Record<string, number | boolean | string>;

/** Defaults of the definition overridden by the instance's stored values. */
export function instanceParams(definition: ComponentDefinition, params: ParamValues): ParamValues {
  return { ...defaultParams(definition), ...params };
}

// Derived parameters are read-only and computed from other parameters. The `formula` text in the
// definition documents them; the computation lives here, keyed by parameter key.
const DERIVED: Record<string, (p: ParamValues) => number> = {
  rollerCount: (p) => Math.floor(num(p, 'length', num(p, 'platformLength', 0)) / num(p, 'rollerPitch', Infinity)),
  sweptRadius: (p) =>
    0.5 * Math.hypot(num(p, 'platformLength', 0), num(p, 'platformWidth', 0)) + num(p, 'sweepClearance', 0),
  entryHeight: (p) => num(p, 'exitHeight', 0) + num(p, 'slope', 0) * num(p, 'length', 0),
};

/** Values of the derived parameters of a definition (keys without a known computation are left out). */
export function derivedValues(definition: ComponentDefinition, params: ParamValues): Record<string, number> {
  const values: Record<string, number> = {};
  for (const p of definition.parameters) {
    const compute = p.derived === true ? DERIVED[p.key] : undefined;
    if (compute !== undefined) values[p.key] = compute(params);
  }
  return values;
}

/** Numeric parameter value, or `fallback` when missing or not a number. */
export function num(params: ParamValues, key: string, fallback: number): number {
  const value = params[key];
  return typeof value === 'number' ? value : fallback;
}
