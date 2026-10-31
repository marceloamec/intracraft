// Support structure and runner guides of straight conveyors (SI units), shared by the 3D models.
// Indicative values from heavy pallet conveyors: feet at most 2 m apart, a cross beam between the
// feet from 650 mm conveying height and diagonal bracing from 850 mm, guide disks on every third
// roller (EUR pallets).

/** Longest bay between two pairs of feet (m). */
export const MAX_SUPPORT_SPACING = 2;
/** Feet sit this far in from the section ends (m). */
export const SUPPORT_INSET = 0.1;
/** Conveying height from which the feet are joined by a cross beam (m). */
export const CROSS_BEAM_FROM = 0.65;
/** Conveying height from which the feet are braced diagonally (m). */
export const DIAGONAL_BRACING_FROM = 0.85;
/** One roller in this many carries guide disks. */
export const GUIDE_DISK_EVERY = 3;
/** Free play between a guide disk and the runner it guides (m). */
export const GUIDE_DISK_CLEARANCE = 0.005;

export type RunnerGuide = 'none' | 'outerOutside' | 'outerInside' | 'centre';

/** Positions of the pairs of feet along a straight section (m from its start), evenly spread. */
export function supportPositions(length: number, inset = SUPPORT_INSET, maxSpacing = MAX_SUPPORT_SPACING): number[] {
  const span = length - 2 * inset;
  if (span <= 0) return [length / 2];
  const bays = Math.max(1, Math.ceil(span / maxSpacing - 1e-9));
  return Array.from({ length: bays + 1 }, (_, i) => inset + (i * span) / bays);
}

/** Bracing a support frame needs at a conveying height (m). */
export function frameBracing(height: number): { crossBeam: boolean; diagonal: boolean } {
  return { crossBeam: height >= CROSS_BEAM_FROM - 1e-9, diagonal: height >= DIAGONAL_BRACING_FROM - 1e-9 };
}

/** Indices of the rollers that carry guide disks: every third one, starting with the first. */
export function guideDiskRollers(count: number, every = GUIDE_DISK_EVERY): number[] {
  return Array.from({ length: Math.ceil(count / every) }, (_, i) => i * every).filter((i) => i < count);
}

/**
 * Lateral positions of the guide disks (m from the conveyor axis) for a pallet whose runners run
 * along travel: outside or inside the two outer runners, or on both sides of the centre runner.
 */
export function guideDiskPositions(
  kind: RunnerGuide,
  palletWidth: number,
  runners: { outer: number; centre: number },
  clearance = GUIDE_DISK_CLEARANCE,
): number[] {
  const half = (() => {
    switch (kind) {
      case 'none':
        return undefined;
      case 'outerOutside':
        return palletWidth / 2 + clearance;
      case 'outerInside':
        return palletWidth / 2 - runners.outer - clearance;
      case 'centre':
        return runners.centre / 2 + clearance;
    }
  })();
  return half === undefined ? [] : [-half, half];
}
