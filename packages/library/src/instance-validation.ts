// Runs the validation rules a definition lists, with the instance's parameter values.
// Until the layout carries a pallet type per line, rules are checked against the EUR pallet.
import type { ComponentDefinition } from './definition-schema';
import { type ParamValues, instanceParams, num } from './params';
import { PALLET_TYPES } from './reference-data';
import {
  type ValidationIssue,
  brakeSpacingTooLong,
  chainFitsBetweenRollers,
  deckTooShort,
  heightsReversed,
  min3RollersUnderPallet,
  noHeightChange,
  pickupTooShort,
  slopeTooLow,
  usableWidthFitsPallet,
} from './validation-rules';

/** Lateral clearance between pallet and guides, per side (m). Indicative. */
export const GUIDE_CLEARANCE = 0.025;

const EUR = PALLET_TYPES.find((p) => p.id === 'eur');

type Rule = (params: ParamValues, along: number, across: number) => ValidationIssue[];

const RULES: Record<string, Rule> = {
  // A chain deck (elevator, S14) has no rollers to count.
  min3RollersUnderPallet: (p, along) =>
    p['deckType'] === 'chains' ? [] : min3RollersUnderPallet(num(p, 'rollerPitch', 0), along),
  usableWidthFitsPallet: (p, _along, across) =>
    usableWidthFitsPallet(num(p, 'usableWidth', num(p, 'platformWidth', 0)), across, GUIDE_CLEARANCE),
  chainFitsBetweenRollers: (p) =>
    chainFitsBetweenRollers(num(p, 'rollerPitch', 0), num(p, 'rollerDiameter', 0), num(p, 'transferChainWidth', 0)),
  heightsReversed: (p) => heightsReversed(num(p, 'lowHeight', 0), num(p, 'conveyorHeight', 0)),
  noHeightChange: (p) => noHeightChange(num(p, 'entryHeight', 0), num(p, 'exitHeight', 0)),
  deckTooShort: (p, along) => deckTooShort(num(p, 'length', 0), along),
  slopeTooLow: (p) => slopeTooLow(num(p, 'slope', 0), num(p, 'rollingResistance', 0)),
  brakeSpacingTooLong: (p, along) => brakeSpacingTooLong(num(p, 'brakeEvery', 1), num(p, 'rollerPitch', 0), along),
  pickupTooShort: (p, along) => pickupTooShort(num(p, 'pickupLength', 0), along),
};

export function validateInstance(definition: ComponentDefinition, params: ParamValues): ValidationIssue[] {
  if (EUR === undefined) return [];
  const values = instanceParams(definition, params);
  const longAlong = values['palletSideAlongTravel'] !== 'short';
  const along = longAlong ? EUR.length : EUR.width;
  const across = longAlong ? EUR.width : EUR.length;
  return definition.validationRules.flatMap((name) => RULES[name]?.(values, along, across) ?? []);
}
