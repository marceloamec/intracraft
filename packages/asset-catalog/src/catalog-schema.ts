// Manifest of third-party 3D models. The .glb files are NOT in git: `pnpm assets:fetch` downloads
// them into library-assets/ and checks size and hash against this manifest.
// Assets have no ports or signals; sizes are real-world metres, Y up. The rigid-body data the models
// carry (mass, centre of mass, inertia) is copied into `physics` (ADR-0015).
import { localizedTextSchema, vec3Schema } from '@intracraft/core-model';
import { z } from 'zod';

export const ASSET_CATEGORIES = [
  'pallets',
  'boxes',
  'containers',
  'drums',
  'racks',
  'vehicles',
  'conveyors',
  'safety',
] as const;

export const assetCategorySchema = z.enum(ASSET_CATEGORIES);

/** Rigid-body data declared by the model (glTF `KHR_physics_rigid_bodies`), in model coordinates. */
export const assetPhysicsSchema = z.object({
  /** Mass of the modelled object (kg). For packaging (boxes, loaded pallets) it is the empty packaging. */
  mass: z.number().positive(),
  centerOfMass: vec3Schema,
  /** Principal moments of inertia about the centre of mass (kg·m²), axes aligned with the model. */
  inertia: vec3Schema,
  /** Density used by the model author (kg/m³), if declared. */
  density: z.number().positive().optional(),
  /** Material families named by the model's surfaces (wood, metal, cardboard…). */
  materials: z.array(z.string().min(1)),
});

export const assetSchema = z.object({
  id: z.string().regex(/^asset\.[a-z]+\.[a-z0-9_]+$/),
  category: assetCategorySchema,
  name: localizedTextSchema,
  tags: z.array(z.string().min(1)),
  /** Path of the .glb relative to library-assets/. */
  file: z.string().regex(/^models\/[a-z]+\/[A-Za-z0-9_]+\.glb$/),
  bytes: z.number().int().positive(),
  sha256: z.string().regex(/^[0-9a-f]{64}$/),
  /** Bounding-box size in metres: x = width, y = height, z = depth. */
  dimensions: z.object({
    x: z.number().positive(),
    y: z.number().positive(),
    z: z.number().positive(),
  }),
  /** Bounding box in model coordinates (m): where the model sits relative to its origin. */
  bounds: z.object({ min: vec3Schema, max: vec3Schema }),
  physics: assetPhysicsSchema.optional(),
  source: z.object({
    dataset: z.string().min(1),
    /** Dataset commit the file was taken from, so the download stays reproducible. */
    revision: z.string().min(1),
    /** File name in the dataset, without extension. */
    name: z.string().min(1),
  }),
  license: z.object({
    id: z.literal('CC-BY-4.0'),
    attribution: z.string().min(1),
    url: z.url(),
  }),
});

export const assetCatalogSchema = z.object({
  schemaVersion: z.literal(1),
  assets: z.array(assetSchema),
});

export type AssetCategory = z.infer<typeof assetCategorySchema>;
export type Asset = z.infer<typeof assetSchema>;
export type AssetPhysics = z.infer<typeof assetPhysicsSchema>;
export type AssetCatalog = z.infer<typeof assetCatalogSchema>;
