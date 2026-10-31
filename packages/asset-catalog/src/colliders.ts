// Collision pieces of the catalogue models (ADR-0017): convex pieces in model coordinates, built by
// scripts/build-catalog.mjs from the colliders each GLB declares. Stored in millimetres to keep the
// file small; read here in metres. Imported only by the physics world (`@intracraft/asset-catalog/colliders`).
import type { Vec3 } from '@intracraft/core-model';
import { z } from 'zod';
import rawColliders from './colliders.json';

const MM = 1000;

export const collidersFileSchema = z.object({
  schemaVersion: z.literal(1),
  unit: z.literal('mm'),
  /** Per asset id: its pieces, each a flat list x0, y0, z0, x1, … of hull corners. */
  assets: z.record(
    z.string(),
    z.array(z.array(z.number().int()).refine((p) => p.length % 3 === 0 && p.length >= 12)).min(1),
  ),
});

const COLLIDERS = collidersFileSchema.parse(rawColliders).assets;

/** Ids of the assets that have collision pieces. */
export function colliderAssetIds(): string[] {
  return Object.keys(COLLIDERS);
}

/** Convex pieces of an asset, each a list of hull corners in model coordinates (m). */
export function assetColliders(assetId: string): Vec3[][] {
  const pieces = COLLIDERS[assetId];
  if (pieces === undefined) throw new Error(`Asset "${assetId}" has no collision pieces`);
  return pieces.map((flat) => {
    const points: Vec3[] = [];
    for (let i = 0; i < flat.length; i += 3) {
      points.push({
        x: (flat[i] ?? 0) / MM,
        y: (flat[i + 1] ?? 0) / MM,
        z: (flat[i + 2] ?? 0) / MM,
      });
    }
    return points;
  });
}
