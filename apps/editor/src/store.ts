// UI state only. Simulation state will arrive as snapshots from the simulation Web Worker.
import { create } from 'zustand';
import type { AssetCategory } from '@intracraft/asset-catalog';
import type { Language } from './i18n/strings';

export type ViewMode = 'perspective' | 'top';
export type SimMode = 'des' | 'physics';

interface EditorState {
  language: Language;
  viewMode: ViewMode;
  simMode: SimMode;
  catalogOpen: boolean;
  catalogCategory: AssetCategory | 'all';
  catalogQuery: string;
  selectedAssetId: string | null;
  setLanguage(language: Language): void;
  toggleView(): void;
  setSimMode(mode: SimMode): void;
  toggleCatalog(): void;
  setCatalogCategory(category: AssetCategory | 'all'): void;
  setCatalogQuery(query: string): void;
  selectAsset(id: string | null): void;
}

export const useEditorStore = create<EditorState>()((set) => ({
  language: 'en',
  viewMode: 'perspective',
  simMode: 'des',
  catalogOpen: false,
  catalogCategory: 'all',
  catalogQuery: '',
  selectedAssetId: null,
  setLanguage: (language) => set({ language }),
  toggleView: () => set((s) => ({ viewMode: s.viewMode === 'perspective' ? 'top' : 'perspective' })),
  setSimMode: (simMode) => set({ simMode }),
  toggleCatalog: () => set((s) => ({ catalogOpen: !s.catalogOpen })),
  setCatalogCategory: (catalogCategory) => set({ catalogCategory }),
  setCatalogQuery: (catalogQuery) => set({ catalogQuery }),
  selectAsset: (selectedAssetId) => set({ selectedAssetId }),
}));
