// Pallets of the 3D run: the 2D pallet record plus its rigid body (pallet + load as one MTBH,
// ADR-0015) and the pose and velocity read after each step.
import { builtInCatalog } from '@intracraft/asset-catalog';
import { assetColliders } from '@intracraft/asset-catalog/colliders';
import type { Vec3 } from '@intracraft/core-model';
import type { PalletType } from '@intracraft/library';
import type { Pallet, SourceNode } from '@intracraft/sim-des';
import type { BodyHandle, Pose, Velocity } from '../backend';
import { solidBoxInertia } from '../mechanics';
import {
  type ModelPhysics,
  type ModelPlacement,
  type Mtbh,
  type MtbhSpec,
  buildMtbh,
  palletConstruction,
} from '../mtbh';
import { type DeckFrame, dot, frameAxis, rotateVec, toLocal } from './frame';

/** Pallet models with rigid-body data in the asset catalogue, by pallet type (ADR-0015). */
const PALLET_MODELS: Record<string, string> = { eur: 'asset.pallets.pallet_block_a07' };
const PALLET_MATERIAL = 'wood';
/** Contact material of a load block travelling without a pallet (a carton stack, S09 §1.1). */
const BLOCK_MATERIAL = 'cardboard';

/** Catalogue models the viewer draws for a pallet, and where they sit in its body frame. */
export interface PalletLook {
  pallet?: { asset: string; placement: ModelPlacement };
  load?: { asset: string; placement: ModelPlacement };
}

/** The rigid body of what a source creates, with the models to draw. */
export interface SourceBody extends Mtbh {
  look: PalletLook;
}

export interface WorldPallet extends Pallet {
  readonly body: BodyHandle;
  readonly mtbh: Mtbh;
  readonly look: PalletLook;
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

const cache = new Map<string, SourceBody>();

/**
 * Rigid body of what a source creates (S09 §1.1, ADR-0015/0017): the pallet model (the model of the
 * pallet type, a catalogue pallet, or a box of the type's size and tare mass when there is none) with
 * a uniform load block of the source's mass and height, a catalogue object or nothing on top, or the
 * load alone.
 */
export function sourceMtbh(node: SourceNode): SourceBody {
  const { item, palletType } = node;
  const key = [
    palletType.id,
    node.loadMass,
    node.loadHeight,
    item.palletModel,
    item.loadKind,
    item.loadAsset,
  ].join('|');
  const cached = cache.get(key);
  if (cached !== undefined) return cached;
  const palletAsset =
    item.palletModel === 'auto'
      ? PALLET_MODELS[palletType.id]
      : item.palletModel === 'none'
        ? undefined
        : item.palletModel;
  const pallet =
    item.palletModel === 'none'
      ? undefined
      : {
          ...palletModel(palletType, palletAsset),
          construction: palletConstruction(palletType),
          material:
            palletAsset === undefined
              ? PALLET_MATERIAL
              : builtInCatalog.get(palletAsset).contactMaterial,
        };
  const spec: MtbhSpec = pallet === undefined ? {} : { pallet };
  if (item.loadKind === 'asset') {
    const asset = builtInCatalog.get(item.loadAsset);
    const physics = asset.physics;
    const model = {
      bounds: asset.bounds,
      mass: physics?.mass ?? 1,
      centerOfMass: physics?.centerOfMass ?? { x: 0, y: asset.dimensions.y / 2, z: 0 },
      inertia: physics?.inertia ?? solidBoxInertia(1, asset.dimensions),
    };
    spec.load = {
      mass: node.loadMass > 0 ? node.loadMass : model.mass,
      model,
      hulls: assetColliders(asset.id),
    };
    spec.material = asset.contactMaterial;
  } else if (item.loadKind === 'block' && node.loadMass > 0 && node.loadHeight > 0) {
    const { min, max } = pallet?.bounds ?? footprintBox(palletType);
    const top = pallet === undefined ? 0 : max.y;
    spec.load = {
      mass: node.loadMass,
      bounds: {
        min: { x: min.x, y: top, z: min.z },
        max: { x: max.x, y: top + node.loadHeight, z: max.z },
      },
      centerOffset: { x: 0, y: 0, z: 0 },
    };
    spec.material = BLOCK_MATERIAL;
  }
  const mtbh = buildMtbh(spec);
  const look: PalletLook = {};
  if (palletAsset !== undefined && pallet !== undefined) {
    look.pallet = { asset: palletAsset, placement: mtbh.placement };
  }
  if (item.loadKind === 'asset' && mtbh.loadPlacement !== undefined) {
    look.load = { asset: item.loadAsset, placement: mtbh.loadPlacement };
  }
  const body = { ...mtbh, look };
  cache.set(key, body);
  return body;
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

function palletModel(type: PalletType, assetId: string | undefined): ModelPhysics {
  if (assetId !== undefined) {
    const asset = builtInCatalog.get(assetId);
    if (asset.physics !== undefined) return { ...asset.physics, bounds: asset.bounds };
  }
  const size: Vec3 = { x: type.length, y: type.height, z: type.width };
  return {
    bounds: footprintBox(type),
    mass: type.tareMass,
    centerOfMass: { x: 0, y: size.y / 2, z: 0 },
    inertia: solidBoxInertia(type.tareMass, size),
  };
}

/** The pallet type's outline: centred footprint, underside at 0. */
function footprintBox(type: PalletType): { min: Vec3; max: Vec3 } {
  return {
    min: { x: -type.length / 2, y: 0, z: -type.width / 2 },
    max: { x: type.length / 2, y: type.height, z: type.width / 2 },
  };
}
