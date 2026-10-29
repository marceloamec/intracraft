// Automatic validation rules (MVP plan §6.7). Each rule returns warnings, never throws.

export interface ValidationIssue {
  rule: string;
  severity: 'warning' | 'error';
  message: string;
}

export const MIN_ROLLERS_UNDER_PALLET = 3;
export const CHAIN_GAP_MARGIN = 0.01; // m

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
