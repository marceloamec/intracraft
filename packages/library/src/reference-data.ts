// Reference data: pallet types and friction pairs. Indicative starting values, not manufacturer data.
import frictionJson from './definitions/friction-pairs.json';
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

export const frictionPairSchema = z.object({
  id: z.string(),
  name: localizedTextSchema,
  mu: z.number(),
  range: z.tuple([z.number(), z.number()]),
});

export type PalletType = z.infer<typeof palletTypeSchema>;
export type FrictionPair = z.infer<typeof frictionPairSchema>;

export const PALLET_TYPES: PalletType[] = z.array(palletTypeSchema).parse(palletTypesJson);
export const FRICTION_PAIRS: FrictionPair[] = z.array(frictionPairSchema).parse(frictionJson);
