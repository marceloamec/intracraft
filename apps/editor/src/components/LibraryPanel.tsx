// Component library: one card per registered definition, grouped by category; drag a card into
// the 3D or 2D view to place it. Scenery objects are placed from the asset catalogue instead (S17).
import { localize } from '@intracraft/core-model';
import type { ComponentDefinition } from '@intracraft/library';
import { type StringKey } from '../i18n/strings';
import { useT } from '../i18n/useT';
import { registry, useLayoutStore } from '../layout-store';
import { useEditorStore } from '../store';
import { ComponentSymbol } from './ComponentSymbol';
import { DEFINITION_MIME } from './Viewport';

type LibraryCategory = Exclude<ComponentDefinition['category'], 'scenery'>;

const CATEGORY_KEYS: Record<LibraryCategory, StringKey> = {
  conveyors: 'catConveyors',
  sensors: 'catSensors',
  mechanics: 'catMechanics',
  flow: 'catFlow',
  pallets: 'catPallets',
  control: 'catControl',
};

export function LibraryPanel() {
  const t = useT();
  const language = useEditorStore((s) => s.language);
  const { beginNewDrag, endDrag } = useLayoutStore.getState();
  const groups = new Map<LibraryCategory, ComponentDefinition[]>();
  for (const d of registry.list()) {
    if (d.category === 'scenery') continue;
    groups.set(d.category, [...(groups.get(d.category) ?? []), d]);
  }

  return (
    <aside className="side-panel library-panel" aria-label={t('componentsTitle')}>
      <h2>{t('componentsTitle')}</h2>
      <p className="hint">{t('dragHint')}</p>
      {[...groups].map(([category, definitions]) => (
        <section key={category}>
          <h3>{t(CATEGORY_KEYS[category])}</h3>
          <ul className="library-list">
            {definitions.map((d) => (
              <li
                key={d.id}
                className="library-card"
                draggable
                title={localize(d.description, language)}
                onDragStart={(e) => {
                  e.dataTransfer.setData(DEFINITION_MIME, d.id);
                  e.dataTransfer.effectAllowed = 'copy';
                  beginNewDrag(d.id);
                }}
                onDragEnd={() => endDrag(false)}
              >
                <ComponentSymbol definitionId={d.id} />
                <span>{localize(d.name, language)}</span>
              </li>
            ))}
          </ul>
        </section>
      ))}
    </aside>
  );
}
