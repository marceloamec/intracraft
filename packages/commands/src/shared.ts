import { z } from 'zod';

const FULL_TURN = 2 * Math.PI;

export const paramValuesSchema = z.record(z.string(), z.union([z.number(), z.boolean(), z.string()]));

/** Yaw within one turn, [0, 2π). */
export function normalizeYaw(yaw: number): number {
  const wrapped = yaw % FULL_TURN;
  return wrapped < 0 ? wrapped + FULL_TURN : wrapped;
}
