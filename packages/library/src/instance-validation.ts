// Runs the validation rules a definition lists, with the instance's parameter values.
// Until the layout carries a pallet type per line, rules are checked against the EUR pallet.
import type { ComponentDefinition } from './definition-schema';
import { type ParamValues, instanceParams, num } from './params';
import { PALLET_TYPES } from './reference-data';
import {
  type ValidationIssue,
  chainFitsBetweenRollers,
  min3RollersUnderPallet,
  usableWidthFitsPallet,
} from './validation-rules';

/** Lateral clearance between pallet and guides, per side (m). Indicative. */
export const GUIDE_CLEARANCE = 0.025;

const EUR = PALLET_TYPES.find((p) => p.id === 'eur');

type Rule = (params: ParamValues, along: number, across: number) => ValidationIssue[];

const RULES: Record<string, Rule> = {
  min3RollersUnderPallet: (p, along) => min3RollersUnderPallet(num(p, 'rollerPitch', 0), along),
  usableWidthFitsPallet: (p, _along, across) =>
    usableWidthFitsPallet(num(p, 'usableWidth', num(p, 'platformWidth', 0)), across, GUIDE_CLEARANCE),
  chainFitsBetweenRollers: (p) =>
    chainFitsBetweenRollers(num(p, 'rollerPitch', 0), num(p, 'rollerDiameter', 0), num(p, 'transferChainWidth', 0)),
};

export function validateInstance(definition: ComponentDefinition, params: ParamValues): ValidationIssue[] {
  if (EUR === undefined) return [];
  const values = instanceParams(definition, params);
  const longAlong = values['palletSideAlongTravel'] !== 'short';
  const along = longAlong ? EUR.length : EUR.width;
  const across = longAlong ? EUR.width : EUR.length;
  return definition.validationRules.flatMap((name) => RULES[name]?.(values, along, across) ?? []);
}
