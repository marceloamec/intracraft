// Results of the current run: statistics and status of one object, and the run warnings.
// Statistic keys come from the definition's `statistics` list (S01, S09).
import type { StringKey } from '../i18n/strings';
import { useT } from '../i18n/useT';
import { useSimStore } from '../sim-store';

const PERCENT = new Set(['utilization', 'occupancy']);
const SECONDS = new Set(['blockedTime', 'emptyTime', 'meanTransitTime', 'waitingTime', 'meanCycleTime', 'occupiedTime']);
const PER_HOUR = new Set(['throughputPerHour']);

/** Statistics of one instance; nothing before the first run. */
export function InstanceStatistics({ instanceId, keys }: { instanceId: string; keys: string[] }) {
  const t = useT();
  const statistics = useSimStore((s) => s.statistics[instanceId]);
  const status = useSimStore((s) => s.run !== 'stopped' ? s.status[instanceId] : undefined);
  if (statistics === undefined) return null;

  return (
    <section className="prop-section">
      <h3>{t('statistics')}</h3>
      {status !== undefined && (
        <div className="prop-row">
          <span>{t('status')}</span>
          <span className={`run-status ${status}`}>{t(`status_${status}` as StringKey)}</span>
        </div>
      )}
      {keys
        .filter((key) => statistics[key] !== undefined)
        .map((key) => (
          <div key={key} className="prop-row">
            <span>{t(`stat_${key}` as StringKey)}</span>
            <output>{formatStatistic(key, statistics[key] ?? 0)}</output>
          </div>
        ))}
    </section>
  );
}

/** Warnings found while building or running the 2D model. */
export function RunWarnings() {
  const t = useT();
  const issues = useSimStore((s) => s.issues);
  const error = useSimStore((s) => s.error);
  if (issues.length === 0 && error === null) return null;
  return (
    <section className="prop-section">
      <h3>{t('runWarnings')}</h3>
      <ul className="issue-list">
        {error !== null && <li className="error">{error}</li>}
        {issues.map((issue, i) => (
          <li key={i} className="warning">
            {issue.message}
          </li>
        ))}
      </ul>
    </section>
  );
}

export function formatStatistic(key: string, value: number): string {
  if (PERCENT.has(key)) return `${(value * 100).toFixed(1)} %`;
  if (SECONDS.has(key)) return `${value.toFixed(1)} s`;
  if (PER_HOUR.has(key)) return `${value.toFixed(1)} /h`;
  return Number.isInteger(value) ? String(value) : value.toFixed(2);
}
