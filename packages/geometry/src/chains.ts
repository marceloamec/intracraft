// Chain strand placement for chain conveyors, chain turntables and transfer tables (SI units).
import { rollerLayout } from './rollers';

/** Distance from the conveyor edge to the outer strand axis (m). Indicative. */
export const CHAIN_EDGE_INSET = 0.1;

/** Lateral strand axis positions, measured from the conveyor axis (m), evenly spread. */
export function chainLayout(width: number, count: number, edgeInset = CHAIN_EDGE_INSET): number[] {
  if (!Number.isInteger(count) || count < 2) throw new Error('chain count must be an integer ≥ 2');
  const span = width - 2 * edgeInset;
  if (!(span > 0)) throw new Error(`width ${width} m leaves no room for strands`);
  return Array.from({ length: count }, (_, i) => -span / 2 + (i * span) / (count - 1));
}

/**
 * Transfer table: positions along the roller section (m from its start) of the liftable chains.
 * Each chain sits in the middle of a gap between two rollers; chains are spread from the first
 * to the last gap so they support the whole pallet.
 */
export function transferChainPositions(length: number, pitch: number, diameter: number, count: number): number[] {
  const rollers = rollerLayout(length, pitch, diameter).positions;
  const gaps = rollers.slice(1).map((x, i) => (x + (rollers[i] ?? x)) / 2);
  if (!Number.isInteger(count) || count < 1 || count > gaps.length) {
    throw new Error(`${count} chains do not fit in ${gaps.length} roller gaps`);
  }
  if (count === 1) return [gaps[Math.floor(gaps.length / 2)] ?? 0];
  return Array.from({ length: count }, (_, i) => gaps[Math.round((i * (gaps.length - 1)) / (count - 1))] ?? 0);
}
