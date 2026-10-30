// Deck frames: where a conveying surface lies in the plant. Local x runs along the deck from its
// entry end, y is up from the conveying surface and z across, as in the instance frame (A04); a
// gravity line's deck also falls along x with its slope.
import type { Vec3 } from '@intracraft/core-model';
import type { Quat } from '../backend';

export interface DeckFrame {
  /** World point of local (0, 0, 0): the deck's x = 0 end, on the conveying surface, on its axis. */
  origin: Vec3;
  /** Turn about +Y (rad), as the instance yaw. */
  yaw: number;
  /** Fall per metre along local x (gravity line); 0 for level decks. */
  slope: number;
}

export const IDENTITY: Quat = { x: 0, y: 0, z: 0, w: 1 };

/** Rotation of the deck: turn about +Y by the yaw, then pitch down along x by the slope. */
export function frameRotation(frame: DeckFrame): Quat {
  return multiplyQuat(yawQuat(frame.yaw), axisQuat({ x: 0, y: 0, z: 1 }, -Math.atan(frame.slope)));
}

export function toWorld(frame: DeckFrame, local: Vec3): Vec3 {
  return add(frame.origin, rotateVec(frameRotation(frame), local));
}

export function toLocal(frame: DeckFrame, world: Vec3): Vec3 {
  return rotateVec(conjugate(frameRotation(frame)), sub(world, frame.origin));
}

/** World direction of a local axis of the deck. */
export function frameAxis(frame: DeckFrame, axis: 'x' | 'y' | 'z'): Vec3 {
  const unit = { x: axis === 'x' ? 1 : 0, y: axis === 'y' ? 1 : 0, z: axis === 'z' ? 1 : 0 };
  return rotateVec(frameRotation(frame), unit);
}

/** Rotation about +Y (right-handed: +X turns toward −Z), as three.js and the editor use it. */
export function yawQuat(yaw: number): Quat {
  return { x: 0, y: Math.sin(yaw / 2), z: 0, w: Math.cos(yaw / 2) };
}

export function axisQuat(axis: Vec3, angle: number): Quat {
  const s = Math.sin(angle / 2);
  return { x: axis.x * s, y: axis.y * s, z: axis.z * s, w: Math.cos(angle / 2) };
}

export function multiplyQuat(a: Quat, b: Quat): Quat {
  return {
    x: a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
    y: a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
    z: a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
    w: a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
  };
}

export function rotateVec(q: Quat, v: Vec3): Vec3 {
  // v' = v + 2w(q × v) + 2 q × (q × v)
  const tx = 2 * (q.y * v.z - q.z * v.y);
  const ty = 2 * (q.z * v.x - q.x * v.z);
  const tz = 2 * (q.x * v.y - q.y * v.x);
  return {
    x: v.x + q.w * tx + (q.y * tz - q.z * ty),
    y: v.y + q.w * ty + (q.z * tx - q.x * tz),
    z: v.z + q.w * tz + (q.x * ty - q.y * tx),
  };
}

export function conjugate(q: Quat): Quat {
  return { x: -q.x, y: -q.y, z: -q.z, w: q.w };
}

export function add(a: Vec3, b: Vec3): Vec3 {
  return { x: a.x + b.x, y: a.y + b.y, z: a.z + b.z };
}

export function sub(a: Vec3, b: Vec3): Vec3 {
  return { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z };
}

export function scale(v: Vec3, k: number): Vec3 {
  return { x: v.x * k, y: v.y * k, z: v.z * k };
}

export function dot(a: Vec3, b: Vec3): number {
  return a.x * b.x + a.y * b.y + a.z * b.z;
}
