// Bottom line: result of the last command (snap link, warnings, rejections) and layout counts.
import { useT } from '../i18n/useT';
import { useLayoutStore } from '../layout-store';

export function StatusBar() {
  const t = useT();
  const status = useLayoutStore((s) => s.status);
  const preview = useLayoutStore((s) => s.preview);
  const layout = useLayoutStore((s) => s.layout);

  const live =
    preview === null
      ? null
      : preview.link === null
        ? t('snapGrid')
        : `${t('snapPort')} ${preview.link.movingPort} → ${preview.link.target.instanceId}.${preview.link.target.portId}`;
  const issue = (preview?.issues ?? status?.issues ?? [])[0];

  return (
    <footer className="statusbar">
      <span>{live ?? (status?.message !== undefined && status.message !== '' ? `${t('snapped')} ${status.message}` : t('shortcutsHint'))}</span>
      {issue !== undefined && <span className={`status-issue ${issue.severity}`}>{issue.message}</span>}
      <span className="status-counts">
        {t('layoutSummary')
          .replace('{objects}', String(layout.instances.length))
          .replace('{connections}', String(layout.connections.length))}
      </span>
    </footer>
  );
}
