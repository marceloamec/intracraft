import { type Action, useActions } from '../actions';
import { LANGUAGES, type Language } from '../i18n/strings';
import { useT } from '../i18n/useT';
import { SIM_SPEEDS, type SimSpeed } from '../sim/protocol';
import { useSimStore } from '../sim-store';
import { type SimMode, useEditorStore } from '../store';
import { Icon } from './icons';

export function Toolbar() {
  const t = useT();
  const a = useActions();
  const { language, simMode, setLanguage, setSimMode } = useEditorStore();
  const speed = useSimStore((s) => s.speed);
  const setSpeed = useSimStore((s) => s.setSpeed);
  const groups: Action[][] = [
    [a.newLayout, a.open, a.save],
    [a.undo, a.redo],
    [a.rotate, a.remove],
    [a.toggleView, a.frameAll],
    [a.components, a.properties, a.controller, a.assets],
    [a.playPause, a.step, a.stop],
  ];

  return (
    <header className="toolbar">
      {groups.map((group, i) => (
        <div key={i} className="tool-group">
          {group.map((action) => (
            <ToolButton key={action.id} action={action} label={t(action.label)} />
          ))}
        </div>
      ))}
      <label>
        {t('speed')}{' '}
        <select value={String(speed)} onChange={(e) => setSpeed(parseSpeed(e.target.value))}>
          {SIM_SPEEDS.map((s) => (
            <option key={s} value={String(s)}>
              {s === 'max' ? t('speedMax') : `×${s}`}
            </option>
          ))}
        </select>
      </label>
      <label>
        {t('simulationMode')}{' '}
        <select value={simMode} onChange={(e) => setSimMode(e.target.value as SimMode)}>
          <option value="des">{t('modeDes')}</option>
          <option value="physics">{t('modePhysics')}</option>
        </select>
      </label>
      <label>
        {t('language')}{' '}
        <select value={language} onChange={(e) => setLanguage(e.target.value as Language)}>
          {LANGUAGES.map((l) => (
            <option key={l} value={l}>
              {l.toUpperCase()}
            </option>
          ))}
        </select>
      </label>
    </header>
  );
}

function parseSpeed(value: string): SimSpeed {
  return SIM_SPEEDS.find((s) => String(s) === value) ?? 1;
}

function ToolButton({ action, label }: { action: Action; label: string }) {
  const title = action.shortcut === undefined ? label : `${label} (${action.shortcut})`;
  return (
    <button
      type="button"
      className="tool-button"
      title={title}
      aria-label={label}
      aria-pressed={action.checked}
      disabled={!action.enabled}
      onClick={action.run}
    >
      {action.icon !== undefined && <Icon name={action.icon} />}
    </button>
  );
}
