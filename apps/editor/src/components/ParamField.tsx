// One input generated from a parameter definition: number (in its display unit), choice list,
// enum, yes/no or text. Values are committed on Enter or when the field loses focus.
import { localize } from '@intracraft/core-model';
import type { ParameterDefinition } from '@intracraft/library';
import { useEditorStore } from '../store';
import { formatNumber, fromDisplay, toDisplay } from '../units-display';

type Value = number | boolean | string;

export function ParamField({
  parameter: p,
  value,
  disabled,
  onCommit,
}: {
  parameter: ParameterDefinition;
  value: Value | undefined;
  disabled: boolean;
  /** Returns false when the command rejected the value. */
  onCommit(value: Value): boolean;
}) {
  const language = useEditorStore((s) => s.language);
  const label = localize(p.label, language);
  const title = localize(p.description, language);
  const unit = p.displayUnit ?? p.unit;

  if (p.derived === true) {
    return (
      <div className="prop-row" title={title}>
        <span>{label}</span>
        <output>
          {typeof value === 'number'
            ? formatNumber(toDisplay(value, p.unit, p.displayUnit), p.displayUnit)
            : '—'}{' '}
          {unit}
        </output>
      </div>
    );
  }

  if (p.type === 'boolean') {
    return (
      <label className="prop-row" title={title}>
        <span>{label}</span>
        <input
          type="checkbox"
          checked={value === true}
          disabled={disabled}
          onChange={(e) => onCommit(e.target.checked)}
        />
      </label>
    );
  }

  if (p.type === 'enum' || p.choices !== undefined) {
    const options =
      p.choices?.map((c) => ({
        value: String(c),
        text: formatNumber(toDisplay(c, p.unit, p.displayUnit), p.displayUnit),
      })) ??
      (p.options ?? []).map((o) => {
        const named = p.optionLabels?.[o];
        return { value: o, text: named === undefined ? o : localize(named, language) };
      });
    return (
      <label className="prop-row" title={title}>
        <span>{label}</span>
        <span className="prop-input">
          <select
            value={String(value)}
            disabled={disabled}
            onChange={(e) =>
              onCommit(p.choices === undefined ? e.target.value : Number(e.target.value))
            }
          >
            {options.map((o) => (
              <option key={o.value} value={o.value}>
                {o.text}
              </option>
            ))}
          </select>
          {p.choices !== undefined && <small>{unit}</small>}
        </span>
      </label>
    );
  }

  if (p.type === 'text') {
    return (
      <label className="prop-row" title={title}>
        <span>{label}</span>
        <input
          key={String(value)}
          defaultValue={String(value ?? '')}
          disabled={disabled}
          onBlur={(e) => {
            if (e.currentTarget.value !== value && !onCommit(e.currentTarget.value))
              e.currentTarget.value = String(value);
          }}
          onKeyDown={blurOnEnter}
        />
      </label>
    );
  }

  const range = [p.min, p.max].every((x) => x !== undefined)
    ? ` [${formatNumber(toDisplay(p.min ?? 0, p.unit, p.displayUnit), p.displayUnit)} … ${formatNumber(toDisplay(p.max ?? 0, p.unit, p.displayUnit), p.displayUnit)}]`
    : '';
  return (
    <NumberInput
      label={label}
      title={`${title}${range}`}
      unit={unit}
      disabled={disabled}
      value={typeof value === 'number' ? toDisplay(value, p.unit, p.displayUnit) : NaN}
      onCommit={(display) => {
        const si = fromDisplay(display, p.unit, p.displayUnit);
        return onCommit(p.type === 'integer' ? Math.round(si) : si);
      }}
    />
  );
}

export function NumberInput({
  label,
  title,
  unit,
  value,
  disabled = false,
  onCommit,
}: {
  label: string;
  title?: string;
  unit?: string | undefined;
  value: number;
  disabled?: boolean;
  onCommit(value: number): boolean;
}) {
  const text = Number.isFinite(value) ? formatNumber(value, unit) : '';
  return (
    <label className="prop-row" title={title}>
      <span>{label}</span>
      <span className="prop-input">
        <input
          key={text}
          type="text"
          inputMode="decimal"
          defaultValue={text}
          disabled={disabled}
          onBlur={(e) => {
            const entered = e.currentTarget.value.trim().replace(',', '.');
            if (entered === text) return;
            const parsed = Number(entered);
            // Restore the stored value when the input is not a number or the command rejects it.
            if (entered === '' || !Number.isFinite(parsed) || !onCommit(parsed))
              e.currentTarget.value = text;
          }}
          onKeyDown={blurOnEnter}
        />
        {unit !== undefined && <small>{unit}</small>}
      </span>
    </label>
  );
}

function blurOnEnter(e: React.KeyboardEvent<HTMLInputElement>) {
  if (e.key === 'Enter') e.currentTarget.blur();
}
