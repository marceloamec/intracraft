// Manifest of third-party 3D models. The .glb files are NOT in git: `pnpm assets:fetch` downloads
// them into library-assets/ and checks size and hash against this manifest.
// Assets are visual only (no physics, ports or signals); sizes are real-world metres, Y up.
import { localizedTextSchema } from '@intracraft/core-model';
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
export type AssetCatalog = z.infer<typeof assetCatalogSchema>;
