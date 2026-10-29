import { type Action, useActions } from '../actions';
import { LANGUAGES, type Language } from '../i18n/strings';
import { useT } from '../i18n/useT';
import { type SimMode, useEditorStore } from '../store';
import { Icon } from './icons';

export function Toolbar() {
  const t = useT();
  const a = useActions();
  const { language, simMode, setLanguage, setSimMode } = useEditorStore();
  const groups: Action[][] = [
    [a.newLayout, a.undo, a.redo],
    [a.rotate, a.remove],
    [a.toggleView],
    [a.components, a.properties, a.assets],
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
