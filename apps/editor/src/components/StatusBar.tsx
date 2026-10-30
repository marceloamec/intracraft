// Bottom line: result of the last command (snap link, warnings, rejections), simulation clock
// while a run exists, and layout counts.
import { useT } from '../i18n/useT';
import { useLayoutStore } from '../layout-store';
import { useSimStore } from '../sim-store';
import { useEditorStore } from '../store';

const STATUS_LABEL = { snap: 'snapped', saved: 'savedAs', opened: 'opened' } as const;

export function StatusBar() {
  const t = useT();
  const status = useLayoutStore((s) => s.status);
  const preview = useLayoutStore((s) => s.preview);
  const layout = useLayoutStore((s) => s.layout);
  const runState = useSimStore((s) => s.run);
  const time = useSimStore((s) => s.time);
  const pallets = useSimStore((s) => s.pallets);
  const physics = useEditorStore((s) => s.simMode === 'physics');

  const live =
    preview === null
      ? null
      : preview.mount !== undefined
        ? preview.mount.hostId === ''
          ? t('notMounted')
          : `${t('mountedOn')} ${preview.mount.hostId} · ${Math.round(preview.mount.s * 1000)} mm`
        : preview.link === null
          ? t('snapGrid')
          : `${t('snapPort')} ${preview.link.movingPort} → ${preview.link.target.instanceId}.${preview.link.target.portId}`;
  const issue = (preview?.issues ?? status?.issues ?? [])[0];

  return (
    <footer className="statusbar">
      <span>
        {live ??
          (status?.message !== undefined && status.message !== ''
            ? `${t(STATUS_LABEL[status.kind ?? 'snap'])} ${status.message}`
            : t('shortcutsHint'))}
      </span>
      {issue !== undefined && (
        <span className={`status-issue ${issue.severity}`}>{issue.message}</span>
      )}
      {runState !== 'stopped' && (
        <span className="status-run">
          {physics && <span className="status-issue warning">{t('physicsNotAvailable')}</span>}
          {t('simTime')} <strong>{formatClock(time)}</strong> ·{' '}
          {t('palletsInModel').replace('{count}', String(pallets))}
        </span>
      )}
      <span className="status-counts">
        {t('layoutSummary')
          .replace('{objects}', String(layout.instances.length))
          .replace('{connections}', String(layout.connections.length))}
      </span>
    </footer>
  );
}

/** Simulation clock as h:mm:ss.s from µs. */
export function formatClock(microseconds: number): string {
  const tenths = Math.floor(microseconds / 100_000);
  const hours = Math.floor(tenths / 36_000);
  const minutes = Math.floor((tenths % 36_000) / 600);
  const seconds = (tenths % 600) / 10;
  return `${hours}:${String(minutes).padStart(2, '0')}:${seconds.toFixed(1).padStart(4, '0')}`;
}
