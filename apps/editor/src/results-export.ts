// Run results as CSV (A09 "export statistics"): one row per object and statistic, in SI units.
// Results are exported separately and never stored in the project file (A04 §5).
import { type Layout, localize } from '@intracraft/core-model';
import type { DefinitionRegistry } from '@intracraft/library';
import type { FlowStatistics } from '@intracraft/sim-des';

const UNITS: Record<string, string> = {
  utilization: 'fraction',
  occupancy: 'fraction',
  blockedTime: 's',
  emptyTime: 's',
  meanTransitTime: 's',
  waitingTime: 's',
  meanCycleTime: 's',
  occupiedTime: 's',
  throughputPerHour: 'pallets/h',
  meanOccupancy: 'pallets',
};

/** CSV text with a header, sorted by layout order; `time` is the simulated time in µs. */
export function resultsCsv(layout: Layout, statistics: FlowStatistics, registry: DefinitionRegistry, time: number): string {
  const rows = [['simulated time (s)', 'object id', 'name', 'type', 'statistic', 'value', 'unit']];
  for (const instance of layout.instances) {
    const values = statistics[instance.id];
    if (values === undefined) continue;
    const type = registry.has(instance.definitionId) ? localize(registry.get(instance.definitionId).name, 'en') : instance.definitionId;
    for (const [key, value] of Object.entries(values)) {
      rows.push([String(time / 1_000_000), instance.id, instance.name, type, key, String(value), UNITS[key] ?? 'count']);
    }
  }
  return `${rows.map((row) => row.map(csvField).join(',')).join('\n')}\n`;
}

function csvField(text: string): string {
  return /[",\n]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text;
}
