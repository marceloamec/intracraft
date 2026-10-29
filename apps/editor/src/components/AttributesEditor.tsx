// Free key/value attributes of an instance (e.g. zone = A, line = 3). Numbers and yes/no are
// recognised from the text; everything else is stored as text.
import type { Attributes } from '@intracraft/core-model';
import { useState } from 'react';
import { useT } from '../i18n/useT';

export function AttributesEditor({
  attributes,
  onChange,
}: {
  attributes: Attributes;
  onChange(attributes: Attributes): boolean;
}) {
  const t = useT();
  const [key, setKey] = useState('');
  const [value, setValue] = useState('');
  const entries = Object.entries(attributes);

  const add = () => {
    const name = key.trim();
    if (name === '') return;
    if (onChange({ ...attributes, [name]: parseValue(value) })) {
      setKey('');
      setValue('');
    }
  };

  return (
    <>
      {entries.length === 0 && <p className="hint">{t('noAttributes')}</p>}
      {entries.map(([name, v]) => (
        <div key={name} className="prop-row attribute-row">
          <span>{name}</span>
          <span className="prop-input">
            <input
              key={String(v)}
              defaultValue={String(v)}
              onBlur={(e) => {
                const next = parseValue(e.currentTarget.value);
                if (next !== v) onChange({ ...attributes, [name]: next });
              }}
              onKeyDown={(e) => {
                if (e.key === 'Enter') e.currentTarget.blur();
              }}
            />
            <button
              type="button"
              className="icon-button"
              title={t('removeAttribute')}
              aria-label={t('removeAttribute')}
              onClick={() => onChange(Object.fromEntries(entries.filter(([k]) => k !== name)))}
            >
              ×
            </button>
          </span>
        </div>
      ))}
      <div className="attribute-add">
        <input placeholder={t('attributeKey')} value={key} onChange={(e) => setKey(e.target.value)} />
        <input
          placeholder={t('attributeValue')}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') add();
          }}
        />
        <button type="button" onClick={add} disabled={key.trim() === ''}>
          {t('add')}
        </button>
      </div>
    </>
  );
}

function parseValue(text: string): number | boolean | string {
  const trimmed = text.trim();
  if (trimmed === 'true' || trimmed === 'false') return trimmed === 'true';
  const number = Number(trimmed.replace(',', '.'));
  return trimmed !== '' && Number.isFinite(number) ? number : trimmed;
}
