// Checks parameter values against the definition schema before a command stores them.
import type { ComponentDefinition, ParameterDefinition, ValidationIssue } from '@intracraft/library';

const CHOICE_TOLERANCE = 1e-9;

export function checkParams(definition: ComponentDefinition, values: Record<string, unknown>): ValidationIssue[] {
  return Object.entries(values).flatMap(([key, value]) => {
    const parameter = definition.parameters.find((p) => p.key === key);
    if (parameter === undefined) return [issue('unknownParam', `${definition.id} has no parameter "${key}"`)];
    if (parameter.derived === true) return [issue('paramReadOnly', `"${key}" is derived and cannot be set`)];
    const problem = checkValue(parameter, value);
    return problem === null ? [] : [problem];
  });
}

function checkValue(p: ParameterDefinition, value: unknown): ValidationIssue | null {
  switch (p.type) {
    case 'number':
    case 'integer':
      if (typeof value !== 'number' || !Number.isFinite(value)) return wrongType(p, 'a number');
      if (p.type === 'integer' && !Number.isInteger(value)) return wrongType(p, 'a whole number');
      if ((p.min !== undefined && value < p.min) || (p.max !== undefined && value > p.max)) {
        return issue('paramOutOfRange', `"${p.key}" = ${value} is outside [${p.min ?? '−∞'}, ${p.max ?? '∞'}] ${p.unit ?? ''}`);
      }
      if (p.choices !== undefined && !p.choices.some((c) => Math.abs(c - value) < CHOICE_TOLERANCE)) {
        return issue('paramNotAChoice', `"${p.key}" must be one of ${p.choices.join(', ')} ${p.unit ?? ''}`);
      }
      return null;
    case 'enum':
      if (typeof value !== 'string') return wrongType(p, 'text');
      if (!(p.options ?? []).includes(value)) {
        return issue('paramNotAnOption', `"${p.key}" must be one of ${(p.options ?? []).join(', ')}`);
      }
      return null;
    case 'boolean':
      return typeof value === 'boolean' ? null : wrongType(p, 'yes/no');
    case 'text':
      return typeof value === 'string' ? null : wrongType(p, 'text');
  }
}

function wrongType(p: ParameterDefinition, expected: string): ValidationIssue {
  return issue('paramWrongType', `"${p.key}" must be ${expected}`);
}

function issue(rule: string, message: string): ValidationIssue {
  return { rule, severity: 'error', message: message.trim() };
}
