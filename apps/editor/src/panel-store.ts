// Which side panels are visible. Kept apart from store.ts, which the asset-catalog work also uses.
import { create } from 'zustand';

interface PanelState {
  showComponents: boolean;
  showProperties: boolean;
  showController: boolean;
  toggleComponents(): void;
  toggleProperties(): void;
  toggleController(): void;
}

export const usePanelStore = create<PanelState>()((set) => ({
  showComponents: true,
  showProperties: true,
  showController: false,
  toggleComponents: () => set((s) => ({ showComponents: !s.showComponents })),
  toggleProperties: () => set((s) => ({ showProperties: !s.showProperties })),
  toggleController: () => set((s) => ({ showController: !s.showController })),
}));
