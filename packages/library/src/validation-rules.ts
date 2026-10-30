// Automatic validation rules (MVP plan §6.7). Each rule returns warnings, never throws.

export interface ValidationIssue {
  rule: string;
  severity: 'warning' | 'error';
  message: string;
}

export const MIN_ROLLERS_UNDER_PALLET = 3;
export const CHAIN_GAP_MARGIN = 0.01; // m
/** Smallest height change of a lift (m); below it a roller conveyor does the job (S13, S14). */
export const MIN_HEIGHT_CHANGE = 0.01;
/** Clearance of a station deck beyond the pallet along travel (m). */
export const MIN_DECK_MARGIN = 0.1;
/** Clearance of the gravity line's pick-up position beyond the pallet (m), S15. */
export const PICKUP_MARGIN = 0.05;

/** At least 3 rollers under the supported pallet side: pitch ≤ side / 3. */
export function min3RollersUnderPallet(pitch: number, palletSupportLength: number): ValidationIssue[] {
  if (pitch <= palletSupportLength / MIN_ROLLERS_UNDER_PALLET) return [];
  return [
    {
      rule: 'min3RollersUnderPallet',
      severity: 'error',
      message: `Roller pitch ${pitch} m leaves fewer than ${MIN_ROLLERS_UNDER_PALLET} rollers under a ${palletSupportLength} m pallet side`,
    },
  ];
}

/** Usable width ≥ pallet width + 2 × guide clearance. */
export function usableWidthFitsPallet(
  usableWidth: number,
  palletWidth: number,
  guideClearance: number,
): ValidationIssue[] {
  const needed = palletWidth + 2 * guideClearance;
  if (usableWidth >= needed) return [];
  return [
    {
      rule: 'usableWidthFitsPallet',
      severity: 'error',
      message: `Usable width ${usableWidth} m is smaller than pallet + clearances (${needed} m)`,
    },
  ];
}

/** Transfer table: gap between rollers (pitch − diameter) ≥ chain width + 10 mm. */
export function chainFitsBetweenRollers(
  pitch: number,
  rollerDiameter: number,
  chainWidth: number,
): ValidationIssue[] {
  const gap = pitch - rollerDiameter;
  if (gap >= chainWidth + CHAIN_GAP_MARGIN) return [];
  return [
    {
      rule: 'chainFitsBetweenRollers',
      severity: 'error',
      message: `Roller gap ${gap.toFixed(3)} m is too small for a ${chainWidth} m chain`,
    },
  ];
}

/** Manual lift table: the operator side must be lower than the conveyor side (S12). */
export function heightsReversed(lowHeight: number, conveyorHeight: number): ValidationIssue[] {
  if (lowHeight < conveyorHeight) return [];
  return [
    {
      rule: 'heightsReversed',
      severity: 'error',
      message: `Low height ${lowHeight} m is not below the conveyor height ${conveyorHeight} m`,
    },
  ];
}

/** Scissor table or elevator whose entry and exit heights are practically equal (S13, S14). */
export function noHeightChange(entryHeight: number, exitHeight: number): ValidationIssue[] {
  if (Math.abs(exitHeight - entryHeight) >= MIN_HEIGHT_CHANGE) return [];
  return [
    {
      rule: 'noHeightChange',
      severity: 'warning',
      message: `Entry and exit heights differ by less than ${MIN_HEIGHT_CHANGE * 1000} mm; a roller conveyor is enough`,
    },
  ];
}

/** A station deck must hold the pallet with some clearance along travel. */
export function deckTooShort(length: number, palletAlong: number): ValidationIssue[] {
  if (length >= palletAlong + MIN_DECK_MARGIN - 1e-9) return [];
  return [
    {
      rule: 'deckTooShort',
      severity: 'warning',
      message: `Deck length ${length} m is shorter than the pallet (${palletAlong} m) plus ${MIN_DECK_MARGIN * 1000} mm`,
    },
  ];
}

/** Gravity line: a pallet at rest only starts rolling when the slope beats the rolling resistance (S15). */
export function slopeTooLow(slope: number, rollingResistance: number): ValidationIssue[] {
  if (slope > rollingResistance) return [];
  return [
    {
      rule: 'slopeTooLow',
      severity: 'error',
      message: `Slope ${slope} is not above the rolling resistance ${rollingResistance}: a pallet at rest does not start`,
    },
  ];
}

/** Gravity line: brake rollers so far apart that a pallet can have none under it (S15). */
export function brakeSpacingTooLong(brakeEvery: number, pitch: number, palletAlong: number): ValidationIssue[] {
  const spacing = brakeEvery * pitch;
  if (spacing <= palletAlong + 1e-9) return [];
  return [
    {
      rule: 'brakeSpacingTooLong',
      severity: 'warning',
      message: `Brake rollers ${spacing.toFixed(3)} m apart: a ${palletAlong} m pallet can have no brake under it`,
    },
  ];
}

/** Gravity line: the pick-up position must hold a pallet with some clearance (S15). */
export function pickupTooShort(pickupLength: number, palletAlong: number): ValidationIssue[] {
  if (pickupLength >= palletAlong + PICKUP_MARGIN - 1e-9) return [];
  return [
    {
      rule: 'pickupTooShort',
      severity: 'error',
      message: `Pick-up length ${pickupLength} m is shorter than the pallet (${palletAlong} m) plus ${PICKUP_MARGIN * 1000} mm`,
    },
  ];
}
