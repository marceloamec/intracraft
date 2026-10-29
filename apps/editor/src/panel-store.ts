// Which side panels are visible. Kept apart from store.ts, which the asset-catalog work also uses.
import { create } from 'zustand';

interface PanelState {
  showComponents: boolean;
  showProperties: boolean;
  toggleComponents(): void;
  toggleProperties(): void;
}

export const usePanelStore = create<PanelState>()((set) => ({
  showComponents: true,
  showProperties: true,
  toggleComponents: () => set((s) => ({ showComponents: !s.showComponents })),
  toggleProperties: () => set((s) => ({ showProperties: !s.showProperties })),
}));
