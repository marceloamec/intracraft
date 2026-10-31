// What a source creates (S09 §1.1): a pallet model and a load, or one of them alone. The sizes are the
// same in 2D and 3D: the footprint is the larger of pallet and load (load turned long side along the
// pallet), the pallet part keeps the height of the pallet type.
import { builtInCatalog } from '@intracraft/asset-catalog';
import type { PalletType } from '@intracraft/library';

export type LoadKind = 'block' | 'none' | 'asset';

export interface SourceItem {
  /** `auto` (model of the pallet type), `none` or a catalogue pallet id. */
  palletModel: string;
  loadKind: LoadKind;
  /** Catalogue object on the pallet, for `loadKind` = asset. */
  loadAsset: string;
  /** Footprint of the whole item, long and short side (m). */
  long: number;
  short: number;
  /** Height of the pallet part (0 without a pallet) and of the load above it (m). */
  palletHeight: number;
  loadHeight: number;
}

export interface SourceItemChoice {
  palletModel: string;
  loadKind: LoadKind;
  loadAsset: string;
  loadHeight: number;
}

/** A catalogue pallet whose footprint differs more than this from its pallet type gets a warning (m). */
export const MODEL_MISMATCH = 0.05;

/** The item of a source; `nothing` when neither pallet nor load was chosen (an empty pallet is sent). */
export function sourceItem(
  type: PalletType,
  choice: SourceItemChoice,
): SourceItem & { nothing: boolean } {
  const nothing = choice.palletModel === 'none' && choice.loadKind === 'none';
  const palletModel = nothing ? 'auto' : choice.palletModel;
  const pallet = palletModel === 'none' ? undefined : { long: type.length, short: type.width };
  let load: { long: number; short: number; height: number } | undefined;
  if (choice.loadKind === 'block') {
    load = { long: type.length, short: type.width, height: choice.loadHeight };
  } else if (choice.loadKind === 'asset') {
    const { x, y, z } = builtInCatalog.get(choice.loadAsset).dimensions;
    load = { long: Math.max(x, z), short: Math.min(x, z), height: y };
  }
  return {
    palletModel,
    loadKind: choice.loadKind,
    loadAsset: choice.loadAsset,
    long: Math.max(pallet?.long ?? 0, load?.long ?? 0),
    short: Math.max(pallet?.short ?? 0, load?.short ?? 0),
    palletHeight: pallet === undefined ? 0 : type.height,
    loadHeight: load?.height ?? 0,
    nothing,
  };
}

/** Footprint mismatch (m) between a catalogue pallet and its pallet type, or 0 for `auto` / `none`. */
export function palletModelMismatch(type: PalletType, palletModel: string): number {
  if (palletModel === 'auto' || palletModel === 'none') return 0;
  const { x, z } = builtInCatalog.get(palletModel).dimensions;
  return Math.max(Math.abs(Math.max(x, z) - type.length), Math.abs(Math.min(x, z) - type.width));
}
