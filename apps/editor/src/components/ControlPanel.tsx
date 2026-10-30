// Controller panel (A08): If/Then rules over the layout's signals. Every change replaces the rule
// list with one `control.setRules` command (undo, log). During a run each signal shows its value.
import type { RuleData } from '@intracraft/core-model';
import { useMemo } from 'react';
import { useT } from '../i18n/useT';
import { registry, useLayoutStore } from '../layout-store';
import { signalCatalog } from '../signals';
import { useSimStore } from '../sim-store';

type Condition = RuleData['when'][number];
type Action = RuleData['then'][number];
type ConditionKind = 'on' | 'off' | 'rising' | 'falling';

const DEFAULT_STOP_MS = 5000;

export function ControlPanel() {
  const t = useT();
  const layout = useLayoutStore((s) => s.layout);
  const run = useLayoutStore((s) => s.run);
  const rules = useMemo(() => layout.rules ?? [], [layout.rules]);
  const catalog = useMemo(() => signalCatalog(layout, registry), [layout]);
  const save = (next: RuleData[]) => run('control.setRules', { rules: next }).ok;
  const update = (index: number, rule: RuleData) => save(rules.map((r, i) => (i === index ? rule : r)));

  const addRule = () => {
    const condition: Condition = { signal: catalog.readable[0] ?? '', edge: 'rising' };
    const action: Action = { signal: catalog.writable[0] ?? '', value: false };
    if (condition.signal === '' || action.signal === '') return;
    save([...rules, { id: freeId(rules), when: [condition], then: [action] }]);
  };

  // The first mounted photocell stops the conveyor it is mounted on, which restarts after a delay.
  const mounted = layout.instances.find((i) => catalog.writable.includes(`${String(i.params['host'])}.run`));
  const addStopTemplate = () => {
    if (mounted === undefined) return;
    const custom = String(mounted.params['outputSignal'] ?? '').trim();
    const sensor = custom === '' ? `${mounted.id}.detected` : custom;
    const conveyor = `${String(mounted.params['host'])}.run`;
    save([
      ...rules,
      {
        id: freeId(rules),
        when: [{ signal: sensor, edge: 'rising' }],
        then: [
          { signal: conveyor, value: false },
          { signal: conveyor, value: true, delayMs: DEFAULT_STOP_MS },
        ],
      },
    ]);
  };

  return (
    <aside className="side-panel control-panel" aria-label={t('controllerTitle')}>
      <h2>{t('controllerTitle')}</h2>
      <p className="hint">{t('controllerHint')}</p>
      <div className="control-buttons">
        <button type="button" onClick={addRule} disabled={catalog.writable.length === 0}>
          {t('addRule')}
        </button>
        <button type="button" onClick={addStopTemplate} disabled={mounted === undefined}>
          {t('stopAtSensorTemplate')}
        </button>
      </div>
      {rules.length === 0 && <p className="hint">{t('noRules')}</p>}
      {rules.map((rule, index) => (
        <RuleCard
          key={rule.id}
          rule={rule}
          readable={catalog.readable}
          writable={catalog.writable}
          onChange={(next) => update(index, next)}
          onRemove={() => save(rules.filter((_, i) => i !== index))}
        />
      ))}
    </aside>
  );
}

function RuleCard({
  rule,
  readable,
  writable,
  onChange,
  onRemove,
}: {
  rule: RuleData;
  readable: string[];
  writable: string[];
  onChange(rule: RuleData): void;
  onRemove(): void;
}) {
  const t = useT();
  const setCondition = (i: number, c: Condition) => onChange({ ...rule, when: rule.when.map((x, j) => (j === i ? c : x)) });
  const setAction = (i: number, a: Action) => onChange({ ...rule, then: rule.then.map((x, j) => (j === i ? a : x)) });

  return (
    <section className="rule-card">
      <header>
        <strong>{rule.id}</strong>
        <button type="button" className="icon-text" onClick={onRemove} aria-label={t('removeRule')}>
          ×
        </button>
      </header>
      <div className="rule-label">{t('ruleIf')}</div>
      {rule.when.map((condition, i) => (
        <div key={i} className="rule-row">
          <SignalSelect value={condition.signal} options={readable} onChange={(signal) => setCondition(i, { ...toCondition(conditionKind(condition), signal) })} />
          <select value={conditionKind(condition)} onChange={(e) => setCondition(i, toCondition(e.target.value as ConditionKind, condition.signal))}>
            <option value="on">{t('isOn')}</option>
            <option value="off">{t('isOff')}</option>
            <option value="rising">{t('rises')}</option>
            <option value="falling">{t('falls')}</option>
          </select>
          <button
            type="button"
            className="icon-text"
            disabled={rule.when.length === 1}
            onClick={() => onChange({ ...rule, when: rule.when.filter((_, j) => j !== i) })}
            aria-label={t('remove')}
          >
            ×
          </button>
        </div>
      ))}
      <button type="button" className="link-button" onClick={() => onChange({ ...rule, when: [...rule.when, { signal: readable[0] ?? '', is: true }] })}>
        + {t('andCondition')}
      </button>
      <div className="rule-label">{t('ruleThen')}</div>
      {rule.then.map((action, i) => (
        <div key={i} className="rule-row">
          <SignalSelect value={action.signal} options={writable} onChange={(signal) => setAction(i, { ...action, signal })} />
          <select value={action.value ? 'on' : 'off'} onChange={(e) => setAction(i, { ...action, value: e.target.value === 'on' })}>
            <option value="on">{t('setOn')}</option>
            <option value="off">{t('setOff')}</option>
          </select>
          <label className="rule-delay" title={t('delayHint')}>
            <input
              key={action.delayMs ?? 0}
              type="text"
              inputMode="numeric"
              defaultValue={String(action.delayMs ?? 0)}
              onBlur={(e) => {
                const ms = Number(e.currentTarget.value.replace(',', '.'));
                if (!Number.isFinite(ms) || ms < 0) e.currentTarget.value = String(action.delayMs ?? 0);
                else if (ms !== (action.delayMs ?? 0)) setAction(i, ms === 0 ? { signal: action.signal, value: action.value } : { ...action, delayMs: ms });
              }}
            />
            <small>ms</small>
          </label>
          <button
            type="button"
            className="icon-text"
            disabled={rule.then.length === 1}
            onClick={() => onChange({ ...rule, then: rule.then.filter((_, j) => j !== i) })}
            aria-label={t('remove')}
          >
            ×
          </button>
        </div>
      ))}
      <button type="button" className="link-button" onClick={() => onChange({ ...rule, then: [...rule.then, { signal: writable[0] ?? '', value: true }] })}>
        + {t('addAction')}
      </button>
    </section>
  );
}

/** Signal list with the live value during a run (● on, ○ off). */
function SignalSelect({ value, options, onChange }: { value: string; options: string[]; onChange(signal: string): void }) {
  const live = useSimStore((s) => (s.run === 'stopped' ? undefined : s.signals[value]));
  const all = options.includes(value) ? options : [value, ...options];
  return (
    <span className="signal-select">
      <span className={`signal-dot ${live === undefined ? '' : live ? 'on' : 'off'}`} aria-hidden="true" />
      <select value={value} onChange={(e) => onChange(e.target.value)}>
        {all.map((signal) => (
          <option key={signal} value={signal}>
            {signal}
          </option>
        ))}
      </select>
    </span>
  );
}

function conditionKind(condition: Condition): ConditionKind {
  if ('edge' in condition) return condition.edge;
  return condition.is ? 'on' : 'off';
}

function toCondition(kind: ConditionKind, signal: string): Condition {
  return kind === 'on' || kind === 'off' ? { signal, is: kind === 'on' } : { signal, edge: kind };
}

function freeId(rules: RuleData[]): string {
  let n = rules.length + 1;
  while (rules.some((r) => r.id === `rule${n}`)) n += 1;
  return `rule${n}`;
}
