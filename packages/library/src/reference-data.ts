// Reference data: pallet types, contact materials and their pairs. Indicative starting values, not
// manufacturer data.
import materialsJson from './definitions/contact-materials.json';
import pairsJson from './definitions/contact-pairs.json';
import palletTypesJson from './definitions/pallet-types.json';
import { localizedTextSchema } from '@intracraft/core-model';
import { z } from 'zod';

export const occupancyBandSchema = z.object({
  /** Height range above the conveying surface (m). */
  fromHeight: z.number(),
  toHeight: z.number(),
  /** Occupied intervals along the pallet length, from its front (m). */
  solid: z.array(z.tuple([z.number(), z.number()])),
});

export const palletTypeSchema = z.object({
  id: z.string(),
  name: localizedTextSchema,
  length: z.number(),
  width: z.number(),
  height: z.number(),
  tareMass: z.number(),
  bottomBoards: z.number().int(),
  /** indicative: from the standard construction; approximate: simplified placeholder to replace. */
  profileStatus: z.enum(['indicative', 'approximate']),
  /**
   * Occupancy seen by a horizontal beam perpendicular to travel. `alongLength`: pallet travels long
   * side first (roller conveyors); `alongWidth`: short side first (chain conveyors).
   */
  occupancy: z.object({
    alongLength: z.array(occupancyBandSchema),
    alongWidth: z.array(occupancyBandSchema),
  }),
});

export const contactMaterialSchema = z.object({
  id: z.string(),
  name: localizedTextSchema,
  description: localizedTextSchema,
});

/** Contact between two materials (S16): one friction coefficient and one restitution, both orders. */
export const contactPairSchema = z.object({
  a: z.string(),
  b: z.string(),
  mu: z.number().nonnegative(),
  range: z.tuple([z.number(), z.number()]),
  restitution: z.number().min(0).max(1),
});

export type PalletType = z.infer<typeof palletTypeSchema>;
export type ContactMaterial = z.infer<typeof contactMaterialSchema>;
export type ContactPair = z.infer<typeof contactPairSchema>;

export const PALLET_TYPES: PalletType[] = z.array(palletTypeSchema).parse(palletTypesJson);
export const CONTACT_MATERIALS: ContactMaterial[] = z.array(contactMaterialSchema).parse(materialsJson);
export const CONTACT_PAIRS: ContactPair[] = z.array(contactPairSchema).parse(pairsJson);

const PAIRS_BY_KEY = new Map(CONTACT_PAIRS.map((p) => [pairKey(p.a, p.b), p]));

/** The pair for materials `a` and `b` in either order; throws if the table has no such pair. */
export function contactPair(a: string, b: string): ContactPair {
  const pair = PAIRS_BY_KEY.get(pairKey(a, b));
  if (pair === undefined) throw new Error(`No contact pair for materials "${a}" and "${b}"`);
  return pair;
}

function pairKey(a: string, b: string): string {
  return a < b ? `${a}|${b}` : `${b}|${a}`;
}
