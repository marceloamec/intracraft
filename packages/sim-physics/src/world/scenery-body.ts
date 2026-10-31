// Rigid body of a scenery object (S17, ADR-0017): a catalogue model placed in the layout or dropped
// into a running world. Its contact shape is the model's convex pieces, its mass properties those of
// the model; `fixed` makes it static, `drop` and `rest` dynamic (allowed to sleep when still).
import { builtInCatalog } from '@intracraft/asset-catalog';
import { assetColliders } from '@intracraft/asset-catalog/colliders';
import type { Vec3 } from '@intracraft/core-model';
import type { BodyDesc } from '../backend';
import { solidBoxInertia } from '../mechanics';
import { yawQuat } from './frame';

export type Placement = 'drop' | 'rest' | 'fixed';

export const PLACEMENTS: readonly Placement[] = ['drop', 'rest', 'fixed'];

/** A scenery object to create: which model, how it enters the run and where. */
export interface SceneryObject {
  asset: string;
  placement: Placement;
  /** Gap between the object's base and its support at the start, for `drop` (m). */
  dropHeight: number;
  /** Point on the support under the model origin (m); its height is the support height. */
  position: Vec3;
  yaw: number;
  /** S16 material; `auto` or absent takes the catalogue's material. */
  material?: string;
}

/** A `rest` object starts this far above its support, so it settles without overlapping (m). */
const REST_GAP = 0.001;
const ZERO: Vec3 = { x: 0, y: 0, z: 0 };

/** Body description of a scenery object; throws for a model that is not in the catalogue. */
export function sceneryBody(object: SceneryObject): BodyDesc {
  const asset = builtInCatalog.get(object.asset);
  const gap =
    object.placement === 'drop' ? object.dropHeight : object.placement === 'rest' ? REST_GAP : 0;
  const pose = {
    position: {
      x: object.position.x,
      y: object.position.y - asset.bounds.min.y + gap,
      z: object.position.z,
    },
    rotation: yawQuat(object.yaw),
  };
  const shapes = assetColliders(object.asset).map((points) => ({
    kind: 'hull' as const,
    points,
    offset: ZERO,
  }));
  const material =
    object.material === undefined || object.material === 'auto'
      ? asset.contactMaterial
      : object.material;
  if (object.placement === 'fixed') return { motion: 'static', pose, shapes, material };

  const physics = asset.physics;
  const { x, y, z } = asset.dimensions;
  const mass = physics?.mass ?? 1;
  const inertia =
    physics !== undefined && physics.inertia.x > 0 && physics.inertia.y > 0 && physics.inertia.z > 0
      ? physics.inertia
      : solidBoxInertia(mass, { x, y, z });
  return {
    motion: 'dynamic',
    pose,
    shapes,
    material,
    massProperties: { mass, centerOfMass: physics?.centerOfMass ?? centre(asset.bounds), inertia },
    ccd: object.placement === 'drop',
    allowSleep: true,
  };
}

function centre({ min, max }: { min: Vec3; max: Vec3 }): Vec3 {
  return { x: (min.x + max.x) / 2, y: (min.y + max.y) / 2, z: (min.z + max.z) / 2 };
}
