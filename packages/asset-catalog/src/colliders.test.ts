import { describe, expect, it } from 'vitest';
import rawCatalog from './catalog.json';
import { createCatalog } from './catalog-queries';
import { assetColliders, colliderAssetIds } from './colliders';

const CATALOG = createCatalog(rawCatalog);
/** Flat pieces are inflated to 5 mm and points are stored to the millimetre. */
const BOUNDS_TOLERANCE = 0.004;
const MAX_PIECES = 48;
const MAX_POINTS = 64;
/** Contact materials a catalogue model can get (S16 §2.1). */
const MODEL_MATERIALS = ['wood', 'plastic', 'cardboard', 'steel-painted', 'rubber'];

describe('collision pieces (colliders.json, ADR-0017)', () => {
  it('has pieces for every catalogue asset and for nothing else', () => {
    expect([...colliderAssetIds()].sort()).toEqual(
      CATALOG.list()
        .map((a) => a.id)
        .sort(),
    );
  });

  it('keeps 1 to 48 pieces per model, each with 4 to 64 corners', () => {
    for (const a of CATALOG.list()) {
      const pieces = assetColliders(a.id);
      expect(pieces.length, a.id).toBeGreaterThanOrEqual(1);
      expect(pieces.length, a.id).toBeLessThanOrEqual(MAX_PIECES);
      for (const piece of pieces) {
        expect(piece.length, a.id).toBeGreaterThanOrEqual(4);
        expect(piece.length, a.id).toBeLessThanOrEqual(MAX_POINTS);
      }
    }
  });

  it('keeps every corner inside the model bounds', () => {
    for (const a of CATALOG.list()) {
      const { min, max } = a.bounds;
      for (const point of assetColliders(a.id).flat()) {
        for (const axis of ['x', 'y', 'z'] as const) {
          expect(point[axis], `${a.id} ${axis}`).toBeGreaterThanOrEqual(
            min[axis] - BOUNDS_TOLERANCE,
          );
          expect(point[axis], `${a.id} ${axis}`).toBeLessThanOrEqual(max[axis] + BOUNDS_TOLERANCE);
        }
      }
    }
  });

  it('gives every model one contact material of the S16 family mapping', () => {
    for (const a of CATALOG.list()) expect(MODEL_MATERIALS, a.id).toContain(a.contactMaterial);
  });

  it('keeps the boards of a wooden pallet apart (gaps between bottom boards stay open)', () => {
    expect(assetColliders('asset.pallets.pallet_block_a07').length).toBeGreaterThan(5);
  });

  it('keeps the openings of a rack: no piece fills more than half of its bounding box', () => {
    const rack = CATALOG.get('asset.racks.rack_large_empty_a01');
    const box = rack.dimensions.x * rack.dimensions.y * rack.dimensions.z;
    for (const piece of assetColliders(rack.id)) expect(volumeOfBox(piece)).toBeLessThan(box / 2);
  });

  it('throws for an unknown asset', () => {
    expect(() => assetColliders('asset.nope.nothing')).toThrow(/no collision pieces/i);
  });
});

function volumeOfBox(points: { x: number; y: number; z: number }[]): number {
  const span = (axis: 'x' | 'y' | 'z') =>
    Math.max(...points.map((p) => p[axis])) - Math.min(...points.map((p) => p[axis]));
  return span('x') * span('y') * span('z');
}
