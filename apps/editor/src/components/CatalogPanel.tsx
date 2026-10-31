// Side panel with the object library: search, category filter, list and 3D preview. A model is
// dragged into the 3D view to place it (S17), with the placement chosen here; during a 3D run it drops
// into the running simulation instead.
import { type Asset, type AssetCategory, builtInCatalog } from '@intracraft/asset-catalog';
import type { Language, StringKey } from '../i18n/strings';
import { useT } from '../i18n/useT';
import { useLayoutStore } from '../layout-store';
import { PLACEMENTS, useEditorStore } from '../store';
import { ASSET_MIME } from './Viewport';
import { AssetPreview } from './AssetPreview';

const MM = 1000;
const MIN_DROP_MM = 50;
const MAX_DROP_MM = 10_000;

export function CatalogPanel() {
  const t = useT();
  const { language, catalogCategory, catalogQuery, selectedAssetId } = useEditorStore();
  const { setCatalogCategory, setCatalogQuery, selectAsset } = useEditorStore();
  const { catalogPlacement, catalogDropHeight, setCatalogPlacement, setCatalogDropHeight } =
    useEditorStore();
  const { beginNewDrag, endDrag } = useLayoutStore.getState();

  const matches = builtInCatalog
    .search(catalogQuery, language)
    .filter((a) => catalogCategory === 'all' || a.category === catalogCategory);
  const selected = selectedAssetId === null ? null : builtInCatalog.get(selectedAssetId);

  return (
    <aside className="catalog">
      <h2>{t('libraryTitle')}</h2>
      <input
        type="search"
        value={catalogQuery}
        placeholder={t('searchAssets')}
        onChange={(e) => setCatalogQuery(e.target.value)}
      />
      <div className="catalog-categories">
        <CategoryButton
          active={catalogCategory === 'all'}
          onClick={() => setCatalogCategory('all')}
        >
          {t('allCategories')}
        </CategoryButton>
        {builtInCatalog.categories().map((category) => (
          <CategoryButton
            key={category}
            active={catalogCategory === category}
            onClick={() => setCatalogCategory(category)}
          >
            {t(categoryKey(category))}
          </CategoryButton>
        ))}
      </div>
      <fieldset className="catalog-placement">
        <legend>{t('placementLabel')}</legend>
        {PLACEMENTS.map((placement) => (
          <label key={placement}>
            <input
              type="radio"
              name="catalog-placement"
              checked={catalogPlacement === placement}
              onChange={() => setCatalogPlacement(placement)}
            />
            {t(`placement_${placement}`)}
          </label>
        ))}
        {catalogPlacement === 'drop' && (
          <label>
            {t('dropHeightLabel')}{' '}
            <input
              type="number"
              min={MIN_DROP_MM}
              max={MAX_DROP_MM}
              step={50}
              value={Math.round(catalogDropHeight * MM)}
              onChange={(e) => setCatalogDropHeight(dropHeight(Number(e.target.value)))}
            />{' '}
            mm
          </label>
        )}
      </fieldset>
      <p className="hint">{t('dragAssetHint')}</p>
      {matches.length === 0 ? (
        <p>{t('noResults')}</p>
      ) : (
        <ul className="catalog-list">
          {matches.map((asset) => (
            <li
              key={asset.id}
              draggable
              onDragStart={(e) => {
                e.dataTransfer.setData(ASSET_MIME, asset.id);
                e.dataTransfer.effectAllowed = 'copy';
                beginNewDrag('scenery.asset', {
                  asset: asset.id,
                  placement: catalogPlacement,
                  dropHeight: catalogDropHeight,
                });
              }}
              onDragEnd={() => endDrag(false)}
            >
              <button
                type="button"
                aria-pressed={asset.id === selectedAssetId}
                onClick={() => selectAsset(asset.id)}
              >
                <span>{localName(asset, language)}</span>
                <small>{formatSize(asset)}</small>
              </button>
            </li>
          ))}
        </ul>
      )}
      <div className="catalog-detail">
        {selected === null ? (
          <p>{t('selectToPreview')}</p>
        ) : (
          <>
            <AssetPreview url={builtInCatalog.urlOf(selected.id)} />
            <strong>{localName(selected, language)}</strong>
            <span>
              {t('sizeLabel')}: {formatSize(selected)}
            </span>
            <span>
              {t('fileSizeLabel')}: {(selected.bytes / 1e6).toFixed(2)} MB
            </span>
            <small>
              {t('attributionLabel')}: {selected.license.attribution}
            </small>
          </>
        )}
      </div>
    </aside>
  );
}

/** Drop height typed in mm, kept within the S17 range (0.05 … 10 m). */
function dropHeight(mm: number): number {
  return Math.min(MAX_DROP_MM, Math.max(MIN_DROP_MM, Number.isFinite(mm) ? mm : MIN_DROP_MM)) / MM;
}

function CategoryButton(props: { active: boolean; onClick(): void; children: string }) {
  return (
    <button type="button" aria-pressed={props.active} onClick={props.onClick}>
      {props.children}
    </button>
  );
}

function categoryKey(category: AssetCategory): StringKey {
  return `category_${category}`;
}

function localName(asset: Asset, language: Language): string {
  return asset.name[language] ?? asset.name.en;
}

/** "1.20 × 0.21 × 0.80 m" (width × height × depth). */
function formatSize({ dimensions }: Asset): string {
  return `${[dimensions.x, dimensions.y, dimensions.z].map((v) => v.toFixed(2)).join(' × ')} m`;
}
