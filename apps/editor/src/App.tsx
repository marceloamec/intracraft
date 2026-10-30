import { useShortcuts } from './actions';
import { CatalogPanel } from './components/CatalogPanel';
import { ControlPanel } from './components/ControlPanel';
import { LibraryPanel } from './components/LibraryPanel';
import { MenuBar } from './components/MenuBar';
import { PropertiesPanel } from './components/PropertiesPanel';
import { StatusBar } from './components/StatusBar';
import { Toolbar } from './components/Toolbar';
import { Viewport } from './components/Viewport';
import { usePanelStore } from './panel-store';
import { useEditorStore } from './store';

export function App() {
  const catalogOpen = useEditorStore((s) => s.catalogOpen);
  const showComponents = usePanelStore((s) => s.showComponents);
  const showProperties = usePanelStore((s) => s.showProperties);
  const showController = usePanelStore((s) => s.showController);
  useShortcuts();

  return (
    <div className="app">
      <MenuBar />
      <Toolbar />
      <div className="workspace">
        {showComponents && <LibraryPanel />}
        <main className="viewport">
          <Viewport />
        </main>
        {showController && <ControlPanel />}
        {showProperties && <PropertiesPanel />}
        {catalogOpen && <CatalogPanel />}
      </div>
      <StatusBar />
    </div>
  );
}
