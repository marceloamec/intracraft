// Roller placement along a conveyor. Pure numbers (SI), shared by the 3D mesh generator,
// the 2D symbol and the physics world builder so all three agree on where rollers are.

export interface RollerLayout {
  count: number;
  /** Roller axis positions along the conveyor, measured from the start (m). */
  positions: number[];
  /** Free gap between two neighbouring rollers: pitch − diameter (m). */
  gap: number;
}

export function rollerLayout(length: number, pitch: number, diameter: number): RollerLayout {
  if (!(pitch > 0) || !(length > 0)) throw new Error('length and pitch must be > 0');
  if (diameter >= pitch) throw new Error(`roller diameter ${diameter} must be < pitch ${pitch}`);
  const count = Math.floor(length / pitch);
  const margin = (length - (count - 1) * pitch) / 2;
  const positions = Array.from({ length: count }, (_, i) => margin + i * pitch);
  return { count, positions, gap: pitch - diameter };
}

/** Number of rollers under a pallet side of length `supportLength` (worst case). */
export function minRollersUnder(supportLength: number, pitch: number): number {
  return Math.floor(supportLength / pitch);
}
