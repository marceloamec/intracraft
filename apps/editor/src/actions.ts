// Editor actions shared by the menu bar, the toolbar and the keyboard shortcuts, so the three
// always do the same thing. Layout changes go through the command session (layout-store).
import { useEffect } from 'react';
import type { StringKey } from './i18n/strings';
import type { IconName } from './components/icons';
import { UNTITLED, useLayoutStore } from './layout-store';
import { usePanelStore } from './panel-store';
import { cameraActions } from './components/Viewport';
import { exportResults, openProject, saveProject } from './project-io';
import { useSimStore } from './sim-store';
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
  const showController = usePanelStore((s) => s.showController);
  const { toggleComponents, toggleProperties, toggleController } = usePanelStore.getState();
  const runState = useSimStore((s) => s.run);
  const hasResults = useSimStore((s) => Object.keys(s.statistics).length > 0);
  const { play, pause, step, stop } = useSimStore.getState();
  const hasObjects = layout.instances.length > 0;

  return {
    newLayout: {
      id: 'newLayout',
      label: 'newLayout',
      icon: 'newLayout',
      enabled: layout.instances.length > 0,
      run: () => {
        select(null);
        run('layout.clear', {});
        useLayoutStore.getState().setProjectInfo({ projectName: UNTITLED });
      },
    },
    open: { id: 'open', label: 'openProject', icon: 'open', shortcut: 'Ctrl+O', enabled: true, run: openProject },
    save: { id: 'save', label: 'saveProject', icon: 'save', shortcut: 'Ctrl+S', enabled: true, run: saveProject },
    exportResults: { id: 'exportResults', label: 'exportResults', enabled: hasResults, run: exportResults },
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
    frameAll: {
      id: 'frameAll',
      label: 'frameAll',
      icon: 'frameAll',
      shortcut: 'F',
      enabled: hasObjects,
      run: () => cameraActions.frameAll?.(),
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
    controller: {
      id: 'controller',
      label: 'controllerTitle',
      icon: 'controller',
      enabled: true,
      checked: showController,
      run: toggleController,
    },
    assets: { id: 'assets', label: 'assets', icon: 'assets', enabled: true, checked: catalogOpen, run: toggleCatalog },
    playPause:
      runState === 'playing'
        ? { id: 'playPause', label: 'pause', icon: 'pause', shortcut: 'Space', enabled: true, run: pause }
        : { id: 'playPause', label: 'play', icon: 'play', shortcut: 'Space', enabled: hasObjects, run: play },
    step: { id: 'step', label: 'step', icon: 'step', enabled: hasObjects, run: step },
    stop: { id: 'stop', label: 'stop', icon: 'stop', enabled: runState !== 'stopped', run: stop },
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
      if ((e.ctrlKey || e.metaKey) && key === 's') saveProject();
      else if ((e.ctrlKey || e.metaKey) && key === 'o') openProject();
      else if ((e.ctrlKey || e.metaKey) && key === 'z' && !e.shiftKey) layoutStore.undo();
      else if ((e.ctrlKey || e.metaKey) && (key === 'y' || (key === 'z' && e.shiftKey))) layoutStore.redo();
      else if (e.ctrlKey || e.metaKey || e.altKey) return;
      else if (key === 'v') useEditorStore.getState().toggleView();
      else if (key === 'f') cameraActions.frameAll?.();
      else if (key === ' ') {
        const sim = useSimStore.getState();
        if (sim.run === 'playing') sim.pause();
        else if (layoutStore.layout.instances.length > 0) sim.play();
      }
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
