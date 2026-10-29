// Side panel with the object library: search, category filter, list and 3D preview.
import { type Asset, type AssetCategory, builtInCatalog } from '@intracraft/asset-catalog';
import type { Language, StringKey } from '../i18n/strings';
import { useT } from '../i18n/useT';
import { useEditorStore } from '../store';
import { AssetPreview } from './AssetPreview';

export function CatalogPanel() {
  const t = useT();
  const { language, catalogCategory, catalogQuery, selectedAssetId } = useEditorStore();
  const { setCatalogCategory, setCatalogQuery, selectAsset } = useEditorStore();

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
      {matches.length === 0 ? (
        <p>{t('noResults')}</p>
      ) : (
        <ul className="catalog-list">
          {matches.map((asset) => (
            <li key={asset.id}>
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
