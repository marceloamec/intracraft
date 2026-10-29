// Internal units are SI (m, kg, s, rad). Convert only at the edges (UI and files).

export const MM_PER_M = 1000;
export const GRAVITY = 9.81; // m/s²

export function mmToM(mm: number): number {
  return mm / MM_PER_M;
}

export function mToMm(m: number): number {
  return m * MM_PER_M;
}

export function degToRad(deg: number): number {
  return (deg * Math.PI) / 180;
}

export function radToDeg(rad: number): number {
  return (rad * 180) / Math.PI;
}
