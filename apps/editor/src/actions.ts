// Editor actions shared by the menu bar, the toolbar and the keyboard shortcuts, so the three
// always do the same thing. Layout changes go through the command session (layout-store).
import { useEffect } from 'react';
import type { StringKey } from './i18n/strings';
import type { IconName } from './components/icons';
import { useLayoutStore } from './layout-store';
import { usePanelStore } from './panel-store';
import { useEditorStore } from './store';

export interface Action {
  id: string;
  label: StringKey;
  icon?: IconName;
  shortcut?: string;
  enabled: boolean;
  checked?: boolean;
  run(): void;
}

export function useActions() {
  const layout = useLayoutStore((s) => s.layout);
  const canUndo = useLayoutStore((s) => s.canUndo);
  const canRedo = useLayoutStore((s) => s.canRedo);
  const selectedId = useLayoutStore((s) => s.selectedId);
  const { run, undo, redo, select, rotateSelected, deleteSelected } = useLayoutStore.getState();
  const viewMode = useEditorStore((s) => s.viewMode);
  const catalogOpen = useEditorStore((s) => s.catalogOpen);
  const { toggleView, toggleCatalog } = useEditorStore.getState();
  const showComponents = usePanelStore((s) => s.showComponents);
  const showProperties = usePanelStore((s) => s.showProperties);
  const { toggleComponents, toggleProperties } = usePanelStore.getState();

  return {
    newLayout: {
      id: 'newLayout',
      label: 'newLayout',
      icon: 'newLayout',
      enabled: layout.instances.length > 0,
      run: () => {
        select(null);
        run('layout.clear', {});
      },
    },
    undo: { id: 'undo', label: 'undo', icon: 'undo', shortcut: 'Ctrl+Z', enabled: canUndo, run: undo },
    redo: { id: 'redo', label: 'redo', icon: 'redo', shortcut: 'Ctrl+Y', enabled: canRedo, run: redo },
    rotate: {
      id: 'rotate',
      label: 'rotate90',
      icon: 'rotate',
      shortcut: 'R',
      enabled: selectedId !== null,
      run: rotateSelected,
    },
    remove: {
      id: 'remove',
      label: 'deleteObject',
      icon: 'remove',
      shortcut: 'Del',
      enabled: selectedId !== null,
      run: deleteSelected,
    },
    toggleView: {
      id: 'toggleView',
      label: viewMode === 'perspective' ? 'viewTop' : 'view3d',
      icon: viewMode === 'perspective' ? 'viewTop' : 'view3d',
      shortcut: 'V',
      enabled: true,
      run: toggleView,
    },
    components: {
      id: 'components',
      label: 'componentsTitle',
      icon: 'components',
      enabled: true,
      checked: showComponents,
      run: toggleComponents,
    },
    properties: {
      id: 'properties',
      label: 'propertiesTitle',
      icon: 'properties',
      enabled: true,
      checked: showProperties,
      run: toggleProperties,
    },
    assets: { id: 'assets', label: 'assets', icon: 'assets', enabled: true, checked: catalogOpen, run: toggleCatalog },
  } satisfies Record<string, Action>;
}

/** Global keyboard shortcuts; ignored while typing in a field. */
export function useShortcuts() {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      if (target !== null && ['INPUT', 'SELECT', 'TEXTAREA'].includes(target.tagName)) return;
      const layoutStore = useLayoutStore.getState();
      const key = e.key.toLowerCase();
      if ((e.ctrlKey || e.metaKey) && key === 'z' && !e.shiftKey) layoutStore.undo();
      else if ((e.ctrlKey || e.metaKey) && (key === 'y' || (key === 'z' && e.shiftKey))) layoutStore.redo();
      else if (e.ctrlKey || e.metaKey || e.altKey) return;
      else if (key === 'v') useEditorStore.getState().toggleView();
      else if (key === 'r') {
        if (layoutStore.isDragging()) layoutStore.rotateDrag();
        else layoutStore.rotateSelected();
      } else if (key === 'delete' || key === 'backspace') layoutStore.deleteSelected();
      else if (key === 'escape') {
        layoutStore.endDrag(false);
        layoutStore.select(null);
      } else return;
      e.preventDefault();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);
}
