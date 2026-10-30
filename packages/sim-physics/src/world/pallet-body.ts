// Pallets of the 3D run: the 2D pallet record plus its rigid body (pallet + load as one MTBH,
// ADR-0015) and the pose and velocity read after each step.
import { builtInCatalog } from '@intracraft/asset-catalog';
import type { Vec3 } from '@intracraft/core-model';
import type { PalletType } from '@intracraft/library';
import type { Pallet, SourceNode } from '@intracraft/sim-des';
import type { BodyHandle, Pose, Velocity } from '../backend';
import { solidBoxInertia } from '../mechanics';
import { type ModelPhysics, type Mtbh, buildMtbh, palletConstruction } from '../mtbh';
import { type DeckFrame, dot, frameAxis, rotateVec, toLocal } from './frame';

/** Pallet models with rigid-body data in the asset catalogue, by pallet type (ADR-0015). */
const PALLET_MODELS: Record<string, string> = { eur: 'asset.pallets.pallet_block_a07' };
const PALLET_MATERIAL = 'wood';

export interface WorldPallet extends Pallet {
  readonly body: BodyHandle;
  readonly mtbh: Mtbh;
  /** Instance id of the object carrying the pallet (the one under its tail, as in 2D). */
  carrier: string;
  pose: Pose;
  velocity: Velocity;
}

/** Footprint of a pallet along a deck: travel coordinates of its front and rear edges (m). */
export interface Extent {
  front: number;
  rear: number;
  /** Lateral offset of the footprint centre from the deck axis (m). */
  lateral: number;
  /** Height of the pallet underside above the conveying surface (m). */
  lift: number;
  /** Pallet speed along the deck's travel direction (m/s). */
  speed: number;
}

const cache = new Map<string, Mtbh>();

/**
 * Rigid body of the pallets a source creates: the catalogue model of the pallet type (or a box of
 * the type's size and tare mass when there is none) with a uniform load block of the source's mass
 * and height on top.
 */
export function sourceMtbh(node: SourceNode): Mtbh {
  const key = `${node.palletType.id}|${node.loadMass}|${node.loadHeight}`;
  const cached = cache.get(key);
  if (cached !== undefined) return cached;
  const model = palletModel(node.palletType);
  const pallet = {
    ...model,
    construction: palletConstruction(node.palletType),
    material: PALLET_MATERIAL,
  };
  const { min, max } = model.bounds;
  const mtbh =
    node.loadMass > 0 && node.loadHeight > 0
      ? buildMtbh({
          pallet,
          load: {
            mass: node.loadMass,
            bounds: {
              min: { x: min.x, y: max.y, z: min.z },
              max: { x: max.x, y: max.y + node.loadHeight, z: max.z },
            },
            centerOffset: { x: 0, y: 0, z: 0 },
          },
        })
      : buildMtbh({ pallet });
  cache.set(key, mtbh);
  return mtbh;
}

/**
 * Where a pallet lies on a deck. `direction` +1 measures travel from the deck's x = 0 end, −1 from
 * its x = length end. The footprint is projected on the deck axis, so a turned pallet reaches further.
 */
export function extentOn(
  pallet: WorldPallet,
  frame: DeckFrame,
  length: number,
  direction: 1 | -1,
): Extent {
  const local = toLocal(frame, pallet.pose.position);
  const along = frameAxis(frame, 'x');
  const bodyX = rotateVec(pallet.pose.rotation, { x: 1, y: 0, z: 0 });
  const bodyZ = rotateVec(pallet.pose.rotation, { x: 0, y: 0, z: 1 });
  const half =
    (Math.abs(dot(bodyX, along)) * pallet.mtbh.length +
      Math.abs(dot(bodyZ, along)) * pallet.mtbh.width) /
    2;
  const centre = direction > 0 ? local.x : length - local.x;
  return {
    front: centre + half,
    rear: centre - half,
    lateral: local.z,
    lift: local.y,
    speed: direction * dot(pallet.velocity.linear, along),
  };
}

function palletModel(type: PalletType): ModelPhysics {
  const assetId = PALLET_MODELS[type.id];
  if (assetId !== undefined) {
    const asset = builtInCatalog.get(assetId);
    if (asset.physics !== undefined) return { ...asset.physics, bounds: asset.bounds };
  }
  const size: Vec3 = { x: type.length, y: type.height, z: type.width };
  return {
    bounds: {
      min: { x: -size.x / 2, y: 0, z: -size.z / 2 },
      max: { x: size.x / 2, y: size.y, z: size.z / 2 },
    },
    mass: type.tareMass,
    centerOfMass: { x: 0, y: size.y / 2, z: 0 },
    inertia: solidBoxInertia(type.tareMass, size),
  };
}
